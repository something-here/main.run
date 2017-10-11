use std::i64;
use std::str::FromStr;

use sincere::Context;
use sincere::Group;

use mon::coll::options::FindOptions;
use mon::oid::ObjectId;

use chrono::Utc;
use chrono::Local;

use common::{Response, Empty};
use middleware;
use model;
use struct_document::StructDocument;
use error::ErrorCode;

#[derive(Deserialize, Debug)]
struct New {
    title: String,
    image: Vec<String>,
    content: String
}

#[derive(Deserialize, Debug)]
struct Update {
    title: String,
    image: Vec<String>,
    content: String
}

pub struct Article;

impl Article {
    pub fn list(context: &mut Context) {
        let page = context.request.get_query("page").unwrap_or("1".to_owned());
        let per_page = context.request.get_query("per_page").unwrap_or("10".to_owned());

        let result = || {
            let page = i64::from_str(&page)?;
            let per_page = i64::from_str(&per_page)?;

            let mut article_find_option = FindOptions::default();

            article_find_option.sort = Some(doc!{
                "_id": (-1)
            });

            article_find_option.limit = Some(per_page);
            article_find_option.skip = Some((page - 1) * per_page);

            let articles = model::Article::find(None, Some(article_find_option))?;

            let articles_count = model::Article::count(None, None)?;

            let mut articles_json = Vec::new();

            for article in articles {
                articles_json.push(json!({
                    "id": article.id.to_hex(),
                    "title": article.title,
                    "image": article.image,
                    "create_at": article.create_at.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string(),
                    "update_at": article.update_at.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string()
                }));
            }

            let return_json = json!({
                "articles": articles_json,
                "count": articles_count
            });

            Ok(Response::success(Some(return_json)))

        };

        match result() {
            Ok(result) => {
                context.response.from_json(result).unwrap();
            },
            Err(err) => {
                context.response.from_json(Response::<Empty>::error(err)).unwrap();
            }
        }

    }

    pub fn detail(context: &mut Context) {
        let article_id = context.request.get_param("id").unwrap();

        let result = || {

            let article_find = doc!{
                "_id": (ObjectId::with_string(&article_id)?)
            };

            let article = model::Article::find_one(Some(article_find), None)?;

            match article {
                None => return Err(ErrorCode(20002).into()),
                Some(doc) => {
                    let return_json = json!({
                        "id": doc.id.to_hex(),
                        "title": doc.title,
                        "image": doc.image,
                        "content": doc.content,
                        "create_at": doc.create_at.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string(),
                        "update_at": doc.update_at.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string()
                    });

                    Ok(Response::success(Some(return_json)))
                }
            }
        };

        match result() {
            Ok(result) => {
                context.response.from_json(result).unwrap();
            },
            Err(err) => {
                context.response.from_json(Response::<Empty>::error(err)).unwrap();
            }
        }
    }

    pub fn new(context: &mut Context) {
        let user_id = context.contexts.get("id").unwrap().as_str().unwrap();

        let request = &context.request;

        let result = || {

            let new_json = request.bind_json::<New>()?;

            let article = model::Article {
                id: ObjectId::new()?,
                title: new_json.title,
                image: new_json.image,
                author_id: ObjectId::with_string(&user_id)?,
                collect_id: Vec::new(),
                content: new_json.content,
                create_at: Utc::now().into(),
                update_at: Utc::now().into()
            };

            article.save(None)?;

            let return_json = json!({
                "article_id": article.id.to_hex()
            });

            Ok(Response::success(Some(return_json)))
        };

        match result() {
            Ok(result) => {
                context.response.from_json(result).unwrap();
            },
            Err(err) => {
                context.response.from_json(Response::<Empty>::error(err)).unwrap();
            }
        }
    }

    pub fn update(context: &mut Context) {
        let article_id = context.request.get_param("id").unwrap();

        let request = &context.request;

        let result = || {

            let update_json = request.bind_json::<Update>()?;

            let article_find = doc!{
                "_id": (ObjectId::with_string(&article_id)?)
            };

            let article = model::Article::find_one(Some(article_find), None)?;

            match article {
                None => return Err(ErrorCode(20002).into()),
                Some(mut doc) => {
                    doc.title = update_json.title;
                    doc.image = update_json.image;
                    doc.content = update_json.content;
                    doc.update_at = Utc::now().into();

                    doc.save(None)?;

                    let return_json = json!({
                        "article_id": article_id
                    });

                    Ok(Response::success(Some(return_json)))
                }
            }
        };

        match result() {
            Ok(result) => {
                context.response.from_json(result).unwrap();
            },
            Err(err) => {
                context.response.from_json(Response::<Empty>::error(err)).unwrap();
            }
        }
    }

    pub fn handle() -> Group {
        let mut group = Group::new("/article");

        group.get("/", Self::list);
        group.get("/{id:[a-z0-9]{24}}", Self::detail);
        group.post("/", Self::new).before(middleware::auth);
        group.put("/{id:[a-z0-9]{24}}", Self::update).before(middleware::auth);

        group
    }
}
