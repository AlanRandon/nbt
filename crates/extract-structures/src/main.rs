use clap::Parser;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

#[derive(Parser)]
pub struct Cli {
    #[clap(subcommand)]
    world: World,
    #[clap(short, long, default_value = "out")]
    output: PathBuf,
}

#[derive(clap::Subcommand)]
pub enum World {
    ComMojang {
        #[clap(long, default_value = LazyLock::force(&COM_MOJANG_DEFAULT).as_os_str())]
        com_mojang: PathBuf,
        world: String,
    },
}

static COM_MOJANG_DEFAULT: LazyLock<std::ffi::OsString> = LazyLock::new(|| {
    if let Some(dir) =
        std::env::home_dir().map(|dir| dir.join(".local/share/mcpelauncher/games/com.mojang"))
    {
        dir.into()
    } else {
        "com.mojang".into()
    }
});

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Fs(#[from] fs_extra::error::Error),
    #[error("database error: {0}")]
    Db(String),
    #[error("output path {0:?} already exists")]
    OutputExists(PathBuf),
}

pub fn open_db(world: World) -> Result<bleveldb::DB, Error> {
    let tempdir = tempfile::tempdir()?;

    log::info!("created tempdir {:?}", tempdir.path());

    match world {
        World::ComMojang { com_mojang, world } => {
            let db_path = com_mojang
                .canonicalize()?
                .join("minecraftWorlds")
                .join(world)
                .join("db");

            log::info!("copying database from {:?}", db_path);

            fs_extra::dir::copy(
                db_path,
                tempdir.path(),
                &fs_extra::dir::CopyOptions::new().content_only(true),
            )?;

            log::info!("database copied to {:?}", tempdir.path());
        }
    }

    log::info!("opening database at {:?}", tempdir.path());

    let options = bleveldb::Options::new();
    options.paranoid_checks(true);
    let db = bleveldb::DB::open(tempdir.path(), &options).map_err(Error::Db)?;

    log::info!("opened database at {:?}", tempdir.path());

    Ok(db)
}

pub fn extract(db: bleveldb::DB, out: &Path) -> Result<(), Error> {
    let mut iter = db.iter(&Default::default());
    iter.seek_to_first();

    for (k, v) in iter {
        if let Some(k) = k.strip_prefix(b"structuretemplate_")
            && let Ok(k) = str::from_utf8(k)
        {
            log::info!("found structure {k:?}");

            let path = out.join(format!("{k}.mcstructure"));
            std::fs::write(&path, &v)?;

            log::info!("extracted structure {k:?} to {path:?}");
        }
    }

    Ok(())
}

pub fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let cli = Cli::parse();

    if std::fs::exists(&cli.output)? {
        anyhow::bail!(Error::OutputExists(cli.output))
    } else {
        std::fs::create_dir(&cli.output)?;
    }

    let db = open_db(cli.world)?;

    extract(db, &cli.output)?;

    Ok(())
}
