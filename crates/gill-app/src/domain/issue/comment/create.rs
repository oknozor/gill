use crate::apub::common::GillApubObject;
use crate::apub::ticket::comment::create::CreateTicketComment;
use crate::domain::issue::comment::IssueComment;
use std::collections::HashSet;

use crate::domain::id::ActivityPubId;
use crate::domain::repository::Repository;
use crate::domain::user::User;
use crate::error::AppResult;
use crate::instance::InstanceHandle;
use activitypub_federation::traits::{Actor, ApubObject};
use chrono::Utc;
use gill_settings::SETTINGS;
use tracing::debug;
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

        let activity_pub_id = format!("{protocol}://{domain}/users/{owner}/repositories/{repository_name}/issues/{number}/comments/{id}");
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
        let hostname = instance.local_instance().hostname();
        let id = format!("https://{hostname}/activity/{uuid}", uuid = Uuid::new_v4());

        let subscribers = issue.get_subscribers_inbox(i64::MAX, 0, db).await?;
        let subscribers: Vec<Url> = subscribers
            .into_iter()
            .filter_map(|inbox| Url::parse(&inbox).ok())
            .collect();

        let mut recipients = HashSet::new();
        let repository_followers = repository.followers(instance).await?;
        let user_followers = author.followers(instance).await?;
        recipients.insert(repository.shared_inbox_or_inbox());
        recipients.extend(repository_followers);
        recipients.extend(subscribers);
        recipients.extend(user_followers);

        let create_event = CreateTicketComment {
            actor: author.activity_pub_id.clone().into(),
            to: vec![
                repository.activity_pub_id.into(),
                repository.followers_url,
                issue.followers_url,
                author.followers_url.clone(),
            ],
            object: comment.into_apub(instance).await?,
            kind: Default::default(),
            id: Url::parse(&id)?,
        };

        debug!(
            "Sending CreateComment event to {:#?}",
            recipients
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
        );

        author
            .send(
                create_event,
                recipients.into_iter().collect(),
                &instance.local_instance,
            )
            .await?;

        Ok(())
    }
}
