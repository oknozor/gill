use crate::apub::common::GillApubObject;
use crate::apub::repository::RepositoryWrapper;
use crate::apub::ticket::comment::create::CreateTicketComment;
use crate::apub::ticket::comment::IssueCommentWrapper;

use crate::apub::user::UserWrapper;
use crate::error::AppError;
use crate::instance::InstanceHandle;
use activitypub_federation::core::object_id::ObjectId;
use activitypub_federation::traits::{Actor, ApubObject};
use chrono::Utc;
use gill_db::repository::issue::comment::IssueComment;
use gill_db::repository::Repository;
use gill_db::user::User;
use gill_db::Insert;
use gill_settings::SETTINGS;
use url::Url;
use uuid::Uuid;

pub struct CreateIssueCommentCommand<'a> {
    pub owner: &'a str,
    pub repository: &'a str,
    pub author_id: i32,
    pub issue_number: i32,
    pub content: String,
}

impl CreateIssueCommentCommand<'_> {
    pub async fn execute(&self, instance: &InstanceHandle) -> Result<(), AppError> {
        let db = instance.database();
        let content = self.content.escape_default().to_string();
        let repository = Repository::by_namespace(self.owner, self.repository, db).await?;
        let issue = repository.get_issue(self.issue_number, db).await?;
        let author = User::by_id(self.author_id, db).await?;
        let id = Uuid::new_v4();
        let protocol = SETTINGS.protocol();
        let domain = &SETTINGS.domain;
        let owner = &self.owner;
        let repository_name = &self.repository;
        let number = &issue.number;

        let activity_pub_id = format!("{protocol}://{domain}/apub/users/{owner}/repositories/{repository_name}/issues/{number}/comments/{id}");

        let comment = IssueComment {
            id,
            activity_pub_id,
            number: issue.number,
            repository_id: repository.id,
            created_by: author.id,
            content,
            media_type: "text/markdown".to_string(),
            attributed_to: author.activity_pub_id.clone(),
            context: issue.activity_pub_id.clone(),
            in_reply_to: issue.activity_pub_id.clone(),
            published: Utc::now().naive_utc(),
        };

        let comment = comment.insert(db).await?;
        let user = UserWrapper::from(author);
        let comment = IssueCommentWrapper::from(comment);
        let repo = RepositoryWrapper::from(repository);
        let hostname = instance.local_instance().hostname();
        let id = format!("https://{hostname}/activity/{uuid}", uuid = Uuid::new_v4());
        let subscribers = issue.get_subscribers(i64::MAX, 0, db).await?;
        let subscribers: Vec<Url> = subscribers
            .into_iter()
            .map(UserWrapper::from)
            .map(|user| user.shared_inbox_or_inbox())
            .collect();

        let to = repo.shared_inbox_or_inbox();
        let mut cc = repo.followers(instance).await?;
        cc.extend(subscribers.clone());

        let mut recipient = vec![to.clone()];
        recipient.extend(cc.clone());
        recipient.extend(subscribers);

        let create_event = CreateTicketComment {
            actor: ObjectId::new(user.activity_pub_id_as_url()?),
            to: vec![to],
            object: comment.into_apub(instance).await?,
            cc,
            kind: Default::default(),
            id: Url::parse(&id)?,
        };

        tracing::debug!(
            "Sending create issue comment activity to user inbox {:?}",
            recipient
        );

        user.send(create_event, recipient.to_owned(), &instance.local_instance)
            .await?;

        Ok(())
    }
}
