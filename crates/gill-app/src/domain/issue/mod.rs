use std::path::is_separator;
use crate::apub::common::{GillApubObject, Source};
use crate::apub::repository::RepositoryWrapper;
use crate::apub::ticket::offer::{ApubTicketOffer, OfferTicket};
use crate::apub::ticket::IssueWrapper;
use crate::apub::user::UserWrapper;
use crate::error::AppError;
use crate::instance::InstanceHandle;
use crate::view::repository::issues::create::CreateIssueForm;
use activitypub_federation::core::object_id::ObjectId;
use activitypub_federation::traits::{Actor, ApubObject};
use chrono::Utc;
use gill_db::repository::issue::{Issue, IssueState};
use gill_db::repository::Repository;
use gill_db::user::User;
use gill_db::Insert;
use gill_settings::SETTINGS;
use url::Url;
use uuid::Uuid;
use crate::apub::ticket::accept::AcceptTicket;

pub mod comment;

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
        let repo = Repository::by_namespace(owner, repository, db).await?;
        let user = UserWrapper::from(user);

        if repo.is_local {
            let number = repo.item_count + 1;
            let protocol = SETTINGS.protocol();
            let domain = &SETTINGS.domain;
            let activity_pub_id = format!(
                "{protocol}://{domain}/apub/users/{owner}/repositories/{repository}/issues/{number}"
            );
            let context = repo.activity_pub_id.to_owned();
            let attributed_to = user.activity_pub_id().to_owned();
            let media_type = "text/markdown".to_owned();
            let followers_url = format!("{activity_pub_id}/followers");
            let team = format!("{activity_pub_id}/team");
            let replies = format!("{activity_pub_id}/replies");
            let dependants = format!("{activity_pub_id}/dependants");
            let dependencies = format!("{activity_pub_id}/dependencies");
            let history = format!("{activity_pub_id}/history");
            let content = self.content.escape_default().to_string();

            let new_issue = Issue {
                repository_id: repo.id,
                opened_by: user.local_id(),
                title: self.title,
                content,
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

            let issue = new_issue.insert(db).await?;

            // Add the author to the list of subscriber
            issue.add_subscriber(user.local_id(), db).await?;

            // If author is not the repository owner, add the owner to
            // the list of subscriber
            if repo.attributed_to != user.activity_pub_id() {
                let owner = User::by_activity_pub_id(&repo.attributed_to, db)
                    .await?
                    .expect("local user must a have an apub identifier");

                issue.add_subscriber(owner.id, db).await?;
            }

            let issue = IssueWrapper::from(issue);
            let repo = RepositoryWrapper::from(repo);
            let hostname = instance.local_instance().hostname();
            let id = format!("https://{hostname}/activity/{uuid}", uuid = Uuid::new_v4());
            let ticket = issue.into_apub(instance).await?;
            let to = repo.followers(instance).await?;
            let recipient = to.clone();
            let create_event = AcceptTicket {
                id: Url::parse(&id)?,
                actor: ObjectId::new(user.activity_pub_id_as_url()?),
                to,
                object: Url::parse(&format!("https://{hostname}/activity/{uuid}", uuid = Uuid::new_v4()))?,
                kind: Default::default(),
                result: ticket.id,
            };

            tracing::debug!(
            "Sending accept issue activity to repository followers inboxes {:?}",
            recipient
        );

            user.send(create_event, recipient, &instance.local_instance)
                .await?;

            Ok(())            
        } else {
            let repository = RepositoryWrapper::from(repo);
            let hostname = &SETTINGS.domain;
            let repository_activity_pub_id = repository.activity_pub_id_as_url()?;
            let to = repository.followers(instance).await?;
            let recipient = to.clone();
            let offer = OfferTicket {
                id: Url::parse(&format!("https://{hostname}/activity/{uuid}", uuid = Uuid::new_v4()))?,
                kind: Default::default(),
                actor: ObjectId::new(user.activity_pub_id_as_url()?),
                to: vec![],
                object: ApubTicketOffer {
                    kind: Default::default(),
                    attributed_to: ObjectId::new(user.activity_pub_id_as_url()?),
                    summary: self.title,
                    media_type: "text/markdown".to_string(),
                    source: Source {
                        content: self.content,
                        media_type: "text/markdown".to_string(),
                    },
                },
                target: ObjectId::new(repository_activity_pub_id),
            };

            user.send(offer, recipient, &instance.local_instance)
                .await
        }
    }
}

