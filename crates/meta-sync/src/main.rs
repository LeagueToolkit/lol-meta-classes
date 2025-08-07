use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use eyre::{Context, Result};
use octocrab::Octocrab;
use tempfile::NamedTempFile;

const CDN_URL: &str = "http://lol.secure.dyn.riotcdn.net/channels/public/bundles";

#[tokio::main]
async fn main() -> Result<()> {
    // Create client (no auth needed for public repos)
    let octocrab = Octocrab::default();

    // List all files in the lol-game-client directories
    println!("Searching for lol-game-client directories in Morilli/riot-manifests:");

    // Option 1: Search for all lol-game-client directories across different regions
    find_lol_game_client_directories(&octocrab, "Morilli", "riot-manifests")
        .await
        .context("Failed to find lol-game-client directories")?;

    // Option 2: If you want to target a specific region, uncomment the lines below:
    // let specific_path = "LoL/EUW1/windows/lol-game-client";
    // println!("\n🎮 Files in {}:", specific_path);
    // list_directory_contents(&octocrab, "Morilli", "riot-manifests", specific_path).await?;

    Ok(())
}

async fn find_lol_game_client_directories(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
) -> Result<()> {
    let mut lol_contents = octocrab
        .repos(owner, repo)
        .get_content()
        .path("LoL/EUW1/macos/lol-game-client")
        .send()
        .await
        .context("Failed to fetch repository contents from GitHub API")?;

    lol_contents.items.sort_by(|a, b| {
        // remove extension from name
        // format: x.y.z.txt

        let a_version = Path::new(&a.name).file_stem().unwrap().to_str().unwrap();
        let b_version = Path::new(&b.name).file_stem().unwrap().to_str().unwrap();

        let a_version = semver::Version::parse(a_version).unwrap();
        let b_version = semver::Version::parse(b_version).unwrap();

        a_version.cmp(&b_version)
    });

    for version_item in lol_contents.items.iter().rev() {
        process_version(version_item).await?;
    }

    Ok(())
}

async fn process_version(version_item: &octocrab::models::repos::Content) -> Result<()> {
    let version = Path::new(&version_item.name)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    println!("Processing version: {}", version);

    let version_manifest_url =
        get_version_manifest_url(version_item.download_url.as_ref().unwrap()).await?;

    let manifest_response = reqwest::get(version_manifest_url).await?;

    let manifest_bytes = manifest_response.bytes().await?;

    let mut manifest_reader = std::io::Cursor::new(manifest_bytes);
    let manifest = rman::Manifest::read(&mut manifest_reader).unwrap();

    // need to match file name with this regex - /.+\/LeagueofLegends
    for file in manifest.files.iter() {
        if !file
            .name
            .eq("LeagueofLegends.app/Contents/MacOS/LeagueofLegends")
        {
            continue;
        }

        // create a temp file
        let temp_file_path = std::env::current_dir()
            .unwrap()
            .join("temp")
            .join(version_item.name.clone());
        fs::create_dir_all(temp_file_path.parent().unwrap())?;
        let mut temp_file = fs::File::create(&temp_file_path)?;

        let version_dump_path = Path::new("dumps").join(format!("{}.json", version));
        fs::create_dir_all(version_dump_path.parent().unwrap())?;

        file.download_all()
            .download(&mut ureq::Agent::new(), CDN_URL, &mut temp_file)
            .map_err(|e| eyre::eyre!("Failed to download file: {}", e))?;

        execute_dumper(&temp_file_path, &version_dump_path)
            .map_err(|e| eyre::eyre!("Failed to dump classes: {}", e))?;
    }

    Ok(())
}

async fn get_version_manifest_url(download_url: &str) -> Result<String> {
    let content = reqwest::get(download_url).await?;
    Ok(content.text().await?)
}

fn execute_dumper(input_path: impl AsRef<Path>, output_path: impl AsRef<Path>) -> Result<()> {
    let mut exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    exe.push("../../target");
    exe.push("release"); // TODO: make this dynamic
    exe.push("dumper");

    println!("Executing dumper: {}", exe.display());
    let output = Command::new(exe)
        .arg(input_path.as_ref())
        .arg("--output")
        .arg(output_path.as_ref())
        .output()
        .map_err(|e| eyre::eyre!("Failed to execute dumper: {}", e))?;

    if !output.status.success() {
        eprintln!("Dumper failed with exit code: {}", output.status);
        eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        return Err(eyre::eyre!("Dumper execution failed"));
    }

    println!("Output: {}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}
