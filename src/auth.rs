use sincere::Context;
use sincere::Group;
use ring::digest::{self, SHA256};

use mon::bson::spec::BinarySubtype;
use mon::oid::ObjectId;

use chrono::Utc;

use common::{Response, Empty};
use util::token;
use error::ErrorCode;

use model;
use struct_document::StructDocument;

#[derive(Deserialize, Debug)]
struct Login {
    username: String,
    password: String,
}

#[derive(Deserialize, Debug)]
struct Logon {
    username: String,
    password: String,
}

pub struct Auth;

impl Auth {
    hand!(login, {|context: &mut Context| {
        let login_json = context.request.bind_json::<Login>()?;

        let actual = digest::digest(&SHA256, login_json.password.as_bytes());

        let doc = doc!{
            "username": (login_json.username),
            "password": (BinarySubtype::Generic, actual.as_ref().to_vec())
        };

        let user = model::User::find_one(Some(doc), None)?;

        match user {
            None => return Err(ErrorCode(20002).into()),
            Some(doc) => {
                let user_id = doc.id.to_string();
                let token = token::generate_token(user_id)?;

                let return_json = json!({
                    "token": token
                });

                Ok(Response::success(Some(return_json)))
            }
        }
    }});

    hand!(logon, {|context: &mut Context| {
        let logon_json = context.request.bind_json::<Logon>()?;

        let doc = doc!{
            "username": (logon_json.username.clone())
        };

        if let Some(_) = model::User::find_one(Some(doc), None)? {
            return Err(ErrorCode(20003).into());
        }

        let actual = digest::digest(&SHA256, logon_json.password.as_bytes());

        let user = model::User {
            id: ObjectId::new()?,
            username: logon_json.username,
            avatar: "".to_owned(),
            role: model::Role::Guest,
            password: actual.as_ref().to_vec(),
            create_at: Utc::now().into(),
            update_at: Utc::now().into()
        };

        user.save(None)?;

        Ok(Response::<Empty>::success(None))
    }});

    pub fn handle() -> Group {
        let mut group = Group::new("/user");

        group.post("/login", Self::login);
        group.post("/logon", Self::logon);

        group
    }
}
