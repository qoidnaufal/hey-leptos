#[cfg(feature = "ssr")]
pub mod ssr {
    use surrealdb::{
        engine::remote::ws::{Client, Ws},
        opt::auth::Root,
        Error, Surreal,
    };

    use crate::auth_model::UserData;

    #[derive(Clone, Debug)]
    pub struct Database {
        pub client: Surreal<Client>,
        pub name_space: String,
        pub db_name: String,
    }

    impl Database {
        pub async fn init() -> Result<Self, Error> {
            let client = Surreal::new::<Ws>("0.0.0.0:8000").await?;
            client
                .signin(Root {
                    username: "root",
                    password: "root",
                })
                .await?;

            client.use_ns("hey").use_db("users").await?;

            Ok(Self {
                client,
                name_space: "hey".to_string(),
                db_name: "users".to_string(),
            })
        }

        pub async fn register_user(&self, uuid: String, new_user: UserData) -> Result<(), Error> {
            let registered_user = self
                .client
                .create::<Option<UserData>>(("user_data", uuid.clone()))
                .content(new_user.clone())
                .await;

            match registered_user {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        }

        pub async fn get_user_by_id(&self, uuid: &str) -> Option<UserData> {
            let get_user = self.client.select(("user_data", uuid)).await;

            match get_user {
                Ok(maybe_user) => maybe_user,
                Err(_) => None,
            }
        }

        pub async fn get_user_by_email(&self, email: &str) -> Result<Option<UserData>, Error> {
            match self
                .client
                .query("SELECT * FROM type::table($table) WHERE email = $email")
                .bind(("table", "user_data"))
                .bind(("email", email))
                .await
            {
                Ok(mut maybe_user) => maybe_user.take::<Option<UserData>>(0),
                Err(err) => Err(err),
            }
        }
    }
}
