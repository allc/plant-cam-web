#[macro_use] extern crate rocket;
use rocket_dyn_templates::{Template, context};
use s3::Region;
use s3::bucket::Bucket;
use s3::serde_types::Object;
use awscreds::Credentials;
use serde::{Serialize, Deserialize};


#[get("/")]
async fn index() -> Template {
    let objects = get_object_list().await;
    if objects.len() < 1 {
        return Template::render("index/no-pictures", context! {});
    }
    let config = get_config();
    let latest_key = &objects.last().unwrap().key;
    let latest_filename = latest_key.split("/").last().unwrap();
    let latest_url = format!("{}/{}", config.r2_bucket_url, latest_key);
    Template::render("index/index", context! {
        latest_filename: latest_filename,
        latest_url: latest_url,
    })
}

#[get("/list")]
async fn list() -> Template {
    let objects = get_object_list().await;
    if objects.len() < 1 {
        return Template::render("index/no-pictures", context! {});
    }
    let config = get_config();
    let object_key_list = objects.iter().map(|object| &object.key).collect::<Vec<&String>>();
    Template::render("list", context! {
        r2_bucket_url: config.r2_bucket_url,
        object_key_list: object_key_list,
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, list])
        .attach(Template::fairing())
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct Config {
    r2_accound_id: String,
    r2_bucket_name: String,
    r2_access_key_id: String,
    r2_secret_access_key: String,
    r2_object_prefix: String,
    r2_bucket_url: String,
}

fn get_config() -> Config {
    confy::load_path("config.toml").expect("Error with config file")
}

fn get_bucket() -> Bucket {
    let config = get_config();
    Bucket::new(
        &config.r2_bucket_name,
        Region::R2 { account_id: config.r2_accound_id },
        Credentials::new(
            Some(&config.r2_access_key_id),
            Some(&config.r2_secret_access_key),
            None, None, None,
        ).expect("Could not initialise S3 credential"),
    ).expect("Could not instantiate the existing bucket")
}

async fn get_object_list() -> Vec<Object> {
    let config = get_config();
    let result = &get_bucket().list(format!("{}pictures/", config.r2_object_prefix), None).await.expect("Unable to get bucket list")[0];
    result.contents.to_owned()
}
