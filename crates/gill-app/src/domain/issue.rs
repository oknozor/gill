use crate::apub::repository::RepositoryWrapper;
use crate::apub::ticket::create::CreateTicket;
use crate::apub::ticket::IssueWrapper;
use crate::apub::user::UserWrapper;
use crate::apub::GillApubObject;
use crate::error::AppError;
use crate::instance::{Instance, InstanceHandle};
use crate::view::repository::issues::create::CreateIssueForm;
use activitypub_federation::core::activity_queue::send_activity;
use activitypub_federation::core::object_id::ObjectId;
use activitypub_federation::traits::ApubObject;
use activitypub_federation::LocalInstance;
use chrono::{NaiveDateTime, Utc};
use gill_db::repository::issue::{Issue, IssueState};
use gill_db::repository::Repository;
use gill_db::user::User;
use gill_settings::SETTINGS;
use sqlx::PgPool;
use url::Url;
use uuid::Uuid;

pub struct CreateIssueCommand {
    title: String,
    content: String,
}

impl From<CreateIssueForm> for CreateIssueCommand {
    fn from(form: CreateIssueForm) -> Self {
        Self {
            title: form.title,
            content: form.content,
        }
    }
}

impl CreateIssueCommand {
    pub async fn execute(
        self,
        repository: &str,
        owner: &str,
        user: User,
        instance: &InstanceHandle,
    ) -> Result<(), AppError> {
        let db = instance.database();
        let repo = Repository::by_namespace(&owner, &repository, &db).await?;
        let number = repo.item_count + 1;
        let activity_pub_id = format!("{}/issues/{}", repo.activity_pub_id, number);
        let context = repo.activity_pub_id.to_owned();
        let attributed_to = user.activity_pub_id.to_owned();
        let media_type = "text/markdown".to_owned();
        let followers_url = format!("{activity_pub_id}/followers");
        let team = format!("{activity_pub_id}/team");
        let replies = format!("{activity_pub_id}/replies");
        let dependants = format!("{activity_pub_id}/dependants");
        let dependencies = format!("{activity_pub_id}/dependencies");
        let history = format!("{activity_pub_id}/history");

        let new_issue = Issue {
            repository_id: repo.id,
            opened_by: user.id,
            title: self.title,
            content: self.content,
            state: IssueState::Open,
            activity_pub_id,
            context,
            attributed_to,
            media_type,
            published: Utc::now().naive_local(),
            followers_url,
            team,
            replies,
            history,
            dependants,
            dependencies,
            resolved_by: None,
            resolved: None,
            number,
            is_local: true,
        };

        let issue = new_issue.insert(&db).await?;

        // Add the author to the list of subscriber
        issue.add_subscriber(user.id, &db).await?;

        // If author is not the repository owner, add the owner to
        // the list of subscriber
        if repo.attributed_to != user.activity_pub_id {
            let owner = User::by_activity_pub_id(&repo.attributed_to, &db)
                .await?
                .expect("local user must a have an apub identifier");

            issue.add_subscriber(owner.id, &db).await?;
        }

        let user = UserWrapper::from(user);
        let issue = IssueWrapper::from(issue);
        let repo = RepositoryWrapper::from(repo);
        let hostname = instance.local_instance().hostname();
        let id = format!("https://{hostname}/activity/{uuid}", uuid = Uuid::new_v4());

        let create_event = CreateTicket {
            actor: ObjectId::new(user.activity_pub_id_as_url()?),
            to: repo.followers(&instance).await?,
            object: issue.into_apub(&instance).await?,
            cc: vec![],
            kind: Default::default(),
            id: Url::parse(&id)?,
        };

        let recipient = create_event.to.to_owned();
        tracing::debug!(
            "Sending create issue activity to user inbox {:?}",
            recipient
        );

        user.send(create_event, recipient.to_owned(), &instance.local_instance)
            .await?;

        Ok(())
    }
}
