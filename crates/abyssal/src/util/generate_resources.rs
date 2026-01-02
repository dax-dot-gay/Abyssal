use rocket::fairing::AdHoc;

use crate::{
    Config,
    models::{RootDirectory, RootDirectoryCollectionExt},
    util::Collection,
};

async fn generate_filesystem(
    config: Config,
    collection: Collection<RootDirectory>,
) -> crate::Result<()> {
    for (name, dir_config) in config.filesystem().directories() {
        if let Some(existing) = collection.by_name(name.clone()).await? {
            if existing.path() != dir_config.path()
                || existing.display_name() != dir_config.display_name()
            {
                let new_root =
                    RootDirectory::new(name, dir_config.display_name(), dir_config.path());
                let _ = collection.save(new_root.with_id(existing.id())).await?;
            }
        } else {
            let new_root = RootDirectory::new(name, dir_config.display_name(), dir_config.path());
            let _ = collection.save(new_root).await?;
        }
    }

    Ok(())
}

pub fn generate_resources() -> AdHoc {
    AdHoc::on_liftoff("Generate configured resources", |rocket| {
        Box::pin(async move {
            let config = rocket.state::<Config>().cloned().unwrap();
            generate_filesystem(config.clone(), Collection::from_rocket(&rocket))
                .await
                .unwrap();
        })
    })
}
