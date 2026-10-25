use std::{collections::BTreeMap, env, error::Error};

use rustic_backend::BackendOptions;
use rustic_core::{Credentials, Repository, RepositoryOptions};

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize Backends
    let aws_key_id = env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID must be set");
    let aws_key_secret =
        env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY must be set");
    let restic_password = env::var("RESTIC_PASSWORD").expect("RESTIC_PASSWORD must be set");
    let endpoint = env::var("RESTIC_AWS_ENDPOINT").expect("RESTIC_AWS_ENDPOINT must be set");
    let bucket = env::var("RESTIC_AWS_BUCKET").expect("RESTIC_AWS_BUCKET must be set");
    let backends = BackendOptions::default()
        .repository("opendal:s3")
        .options(BTreeMap::from([
            ("bucket".to_string(), bucket),
            ("endpoint".to_string(), endpoint),
            ("region".to_string(), "us-east-1".to_string()),
            ("secret_access_key".to_string(), aws_key_secret),
            ("access_key_id".to_string(), aws_key_id),
        ]))
        .to_backends()?;

    // Open repository
    let repo_opts = RepositoryOptions::default();

    let repo = Repository::new(&repo_opts, &backends)?
        .open(&Credentials::password(restic_password))?
        .to_indexed()?;

    let snapshots = repo.get_all_snapshots()?;

    for item in snapshots {
        println!("{item:?} ");
    }
    Ok(())
}
