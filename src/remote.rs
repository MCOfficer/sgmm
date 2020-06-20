pub mod steam {
    use anyhow::Result;

    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct WorkshopItemInfoResponseList {
        response: WorkshopItemInfoResponse,
    }

    #[derive(Deserialize)]
    pub struct WorkshopItemInfoResponse {
        publishedfiledetails: Vec<WorkshopItemInfo>,
    }

    #[derive(Debug, Deserialize)]
    pub struct WorkshopItemInfo {
        file_url: String,
        title: String,
        file_size: u32,
    }

    pub fn retrieve_info(item_id: u32, verbose: bool) -> Result<WorkshopItemInfo> {
        let res = ureq::post(
            "https://api.steampowered.com/ISteamRemoteStorage/GetPublishedFileDetails/v1/",
        )
        .send_form(&[
            ("itemcount", "1"),
            ("publishedfileids[0]", &item_id.to_string()),
        ]);

        if verbose {
            println!("Response: {:#?}", res);
        }

        let mut list: WorkshopItemInfoResponseList = res.into_json_deserialize()?;
        list.response
            .publishedfiledetails
            .pop()
            .ok_or_else(|| anyhow!("Got 0 results from steam!"))
    }
}

pub mod steamworkshopdownloader_io {
    use anyhow::Result;
    use serde::Deserialize;

    const BASE_URL: &str = "https://api.steamworkshopdownloader.io/api/";

    #[derive(Debug, Deserialize)]
    pub struct DownloadRequestResponse {
        uuid: String,
    }

    pub fn request_transfer(item_id: u32, verbose: bool) -> Result<DownloadRequestResponse> {
        let body = format!("{{\"publishedFileId\":{},\"collectionId\":null,\"extract\":true,\"hidden\":false,\"direct\":false,\"autodownload\":true}}", item_id);
        let res = ureq::post(&format!("{}download/request", BASE_URL)).send_bytes(body.as_bytes());

        if verbose {
            println!("Response: {:#?}", res);
        }
        Ok(res.into_json_deserialize::<DownloadRequestResponse>()?)
    }
}

pub mod steamworkshop_download {
    use anyhow::Result;
    use regex::Regex;

    pub fn request_download(item_id: u32, verbose: bool) -> Result<String> {
        let res = ureq::post("http://steamworkshop.download/online/steamonline.php")
            .timeout_read(3_000)
            .send_form(&[("app", "281990"), ("item", &item_id.to_string())])
            .into_string()?;

        if verbose {
            println!("Response: {:#?}", res);
        }

        let re = Regex::new(&format!("http.*?{}.zip", item_id)).unwrap();
        match re.captures(&res) {
            None => {
                println!("Website returned: {}", res);
                Err(anyhow!("Failed to get download link"))
            }
            Some(captures) => Ok(String::from(captures.get(0).unwrap().as_str())),
        }
    }
}
