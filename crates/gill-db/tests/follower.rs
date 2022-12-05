use gill_db::user::User;
use speculoos::prelude::*;
use sqlx::PgPool;

#[sqlx::test(fixtures("base"))]
async fn should_get_user_by_email(db: PgPool) -> eyre::Result<()> {
    let alice = User::by_email("alice@wonder.land", &db).await?;

    let followers: Vec<String> = alice
        .get_followers(10, 0, &db)
        .await?
        .into_iter()
        .map(|user| user.username)
        .collect();

    assert_that!(followers).is_equal_to(vec!["tom".to_string(), "jerry".to_string()]);

    Ok(())
}
