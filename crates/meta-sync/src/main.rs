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
    let lol_contents = octocrab
        .repos(owner, repo)
        .get_content()
        .path("LoL/EUW1/macos/lol-game-client")
        .send()
        .await
        .context("Failed to fetch repository contents from GitHub API")?;

    for version_item in lol_contents.items {
        process_version(&version_item).await?;
    }

    Ok(())
}

async fn process_version(version_item: &octocrab::models::repos::Content) -> Result<()> {
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
        let mut temp_file = NamedTempFile::new().unwrap();

        file.download_all()
            .download(&mut ureq::Agent::new(), CDN_URL, &mut temp_file)
            .map_err(|e| eyre::eyre!("Failed to download file: {}", e))?;

        let classes = dumper::dump_classes_from_file(temp_file.path())
            .map_err(|e| eyre::eyre!("Failed to dump classes: {}", e))?;

        println!("{}", classes);
    }

    Ok(())
}

async fn get_version_manifest_url(download_url: &str) -> Result<String> {
    let content = reqwest::get(download_url).await?;
    Ok(content.text().await?)
}
