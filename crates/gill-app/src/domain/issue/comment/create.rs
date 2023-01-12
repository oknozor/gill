use crate::apub::common::GillApubObject;
use crate::apub::ticket::comment::create::CreateTicketComment;
use crate::domain::issue::comment::IssueComment;

use crate::domain::id::ActivityPubId;
use crate::domain::repository::Repository;
use crate::domain::user::User;
use crate::error::AppResult;
use crate::instance::InstanceHandle;
use activitypub_federation::traits::{Actor, ApubObject};
use chrono::Utc;
use gill_settings::SETTINGS;
use url::Url;
use uuid::Uuid;

pub struct CreateIssueCommentCommand<'a> {
    pub owner: &'a str,
    pub repository: &'a str,
    pub author_id: i32,
    pub issue_number: i32,
    pub content: &'a str,
}

impl CreateIssueCommentCommand<'_> {
    pub async fn execute(&self, instance: &InstanceHandle) -> AppResult<()> {
        let db = instance.database();
        let content = self.content.escape_default().to_string();
        let repository = Repository::by_namespace(self.owner, self.repository, db).await?;
        let issue = repository.issue_by_number(self.issue_number, db).await?;
        let author = User::by_id(self.author_id, db).await?;
        let id = Uuid::new_v4();
        let protocol = SETTINGS.protocol();
        let domain = &SETTINGS.domain;
        let owner = &self.owner;
        let repository_name = &self.repository;
        let number = &issue.number;

        let activity_pub_id = format!("{protocol}://{domain}/apub/users/{owner}/repositories/{repository_name}/issues/{number}/comments/{id}");
        let activity_pub_id = ActivityPubId::try_from(activity_pub_id)?;

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
            in_reply_to: issue.activity_pub_id.clone().into(),
            published: Utc::now().naive_utc(),
        };

        let comment = comment.save(db).await?;
        let user = author;
        let hostname = instance.local_instance().hostname();
        let id = format!("https://{hostname}/activity/{uuid}", uuid = Uuid::new_v4());
        let subscribers = issue.get_subscribers_inbox(i64::MAX, 0, db).await?;
        let subscribers: Vec<Url> = subscribers
            .into_iter()
            .filter_map(|inbox| Url::parse(&inbox).ok())
            .collect();

        let to = repository.shared_inbox_or_inbox();
        let mut cc = repository.followers(instance).await?;
        cc.extend(subscribers.clone());

        let mut recipient = vec![to.clone()];
        recipient.extend(cc.clone());
        recipient.extend(subscribers);

        let create_event = CreateTicketComment {
            actor: user.activity_pub_id.clone().into(),
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
