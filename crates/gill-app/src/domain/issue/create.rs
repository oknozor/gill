use crate::apub::common::{GillApubObject, Source};
use crate::apub::ticket::accept::AcceptTicket;
use crate::apub::ticket::offer::{ApubTicketOffer, OfferTicket};
use crate::domain::id::ActivityPubId;
use crate::domain::issue::{Issue, IssueState};
use crate::domain::repository::Repository;
use crate::domain::user::User;
use crate::error::AppResult;
use crate::instance::InstanceHandle;
use crate::view::repository::issues::create::CreateIssueForm;

use activitypub_federation::traits::ApubObject;
use chrono::Utc;
use gill_settings::SETTINGS;
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
    ) -> AppResult<()> {
        let db = instance.database();
        let repo = Repository::by_namespace(owner, repository, db).await?;

        if repo.is_local {
            let number = repo.item_count + 1;
            let protocol = SETTINGS.protocol();
            let domain = &SETTINGS.domain;
            let activity_pub_id = format!(
                "{protocol}://{domain}/users/{owner}/repositories/{repository}/issues/{number}"
            );
            let context = repo.activity_pub_id.clone();
            let _attributed_to = user.activity_pub_id.clone();
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
                activity_pub_id: ActivityPubId::try_from(activity_pub_id)?,
                context,
                attributed_to: user.activity_pub_id.clone(),
                media_type,
                published: Utc::now().naive_local(),
                followers_url: Url::parse(&followers_url)?,
                team: Url::parse(&team)?,
                replies: Url::parse(&replies)?,
                history: Url::parse(&history)?,
                dependants: Url::parse(&dependants)?,
                dependencies: Url::parse(&dependencies)?,
                resolved_by: None,
                resolved: None,
                number,
                is_local: true,
            };

            let issue = new_issue.save(db).await?;

            // Add the author to the list of subscriber
            issue.add_subscriber(user.local_id(), db).await?;

            // If author is not the repository owner, add the owner to
            // the list of subscriber
            let attributed_to = repo.attributed_to.to_string();
            if attributed_to != issue.attributed_to.to_string() {
                let owner = User::by_activity_pub_id(&attributed_to, db).await?;
                issue.add_subscriber(owner.id, db).await?;
            }

            let hostname = instance.local_instance().hostname();
            let id = format!("https://{hostname}/activity/{uuid}", uuid = Uuid::new_v4());
            let ticket = issue.into_apub(instance).await?;
            let to = repo.followers(instance).await?;
            let recipient = to.clone();
            let accept_ticket = AcceptTicket {
                id: Url::parse(&id)?,
                actor: repo.activity_pub_id.clone().into(),
                to,
                object: Url::parse(&format!(
                    "https://{hostname}/activity/{uuid}",
                    uuid = Uuid::new_v4()
                ))?,
                kind: Default::default(),
                result: ticket.id,
            };

            repo.send(accept_ticket, recipient, &instance.local_instance)
                .await?;

            Ok(())
        } else {
            let repository = repo;
            let hostname = &SETTINGS.domain;
            let attributed_to = user.activity_pub_id.clone().into();
            let actor = user.activity_pub_id.clone().into();

            let repository_activity_pub_id = repository.activity_pub_id;
            let offer = OfferTicket {
                id: Url::parse(&format!(
                    "https://{hostname}/activity/{uuid}",
                    uuid = Uuid::new_v4()
                ))?,
                kind: Default::default(),
                actor,
                to: vec![repository_activity_pub_id.clone().into()],
                object: ApubTicketOffer {
                    kind: Default::default(),
                    attributed_to,
                    summary: self.title,
                    media_type: "text/markdown".to_string(),
                    source: Source {
                        content: self.content,
                        media_type: "text/markdown".to_string(),
                    },
                },
                target: repository_activity_pub_id.clone().into(),
            };

            user.send(offer, vec![repository.inbox_url], &instance.local_instance)
                .await
        }
    }
}
