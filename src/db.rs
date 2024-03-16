use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Error, Surreal,
};

#[derive(Clone, Debug)]
pub struct Database {
    pub client: Surreal<Client>,
    #[allow(dead_code)]
    name_space: &'static str,
    #[allow(dead_code)]
    db_name: &'static str,
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

        client.use_ns("admin").use_db("hey!").await?;

        Ok(Self {
            client,
            name_space: "admin",
            db_name: "hey!",
        })
    }
}

pub trait IntoDb {
    fn insert_into_db(
        &self,
        pool: &Database,
    ) -> impl std::future::Future<Output = Result<(), surrealdb::Error>> + Send;
}
