pub fn get_download_link(item_id: u32, verbose: bool) -> String {
    println!("Attempting download via steamworkshop.download");
    match steamworkshop_download::request_download(item_id, verbose) {
        Ok(download_link) => {
            if verbose {
                println!("Got download link: {}", download_link);
            }
            return download_link;
        }
        Err(e) => {
            yellow_ln!("Failed to request download: {}", e);
        }
    };

    println!("Attempting download via steamworkshopdownloader.io");
    match steamworkshopdownloader_io::request_transfer(item_id, verbose) {
        Ok(download_res) => {
            if verbose {
                println!("Got transfer response: {:#?}", download_res);
            }
            return format!(
                "https://api.steamworkshopdownloader.io/api/download/transmit?uuid={}",
                download_res.uuid
            );
        }
        Err(e) => {
            red_ln!("Failed to request download: {}", e);
            panic!()
        }
    };
}

pub mod steam {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct WorkshopItemInfoResponseList {
        response: WorkshopItemInfoResponse,
    }

    #[derive(Debug, Deserialize)]
    pub struct WorkshopItemInfoResponse {
        publishedfiledetails: Vec<WorkshopItemInfo>,
    }

    #[derive(Debug, Deserialize)]
    pub struct WorkshopItemInfo {
        pub file_url: String,
        pub title: String,
        pub file_size: usize,
    }

    pub fn retrieve_info(item_id: u32, verbose: bool) -> WorkshopItemInfo {
        println!("Fetching info");
        let res = ureq::post(
            "https://api.steampowered.com/ISteamRemoteStorage/GetPublishedFileDetails/v1/",
        )
        .send_form(&[
            ("itemcount", "1"),
            ("publishedfileids[0]", &item_id.to_string()),
        ]);

        let mut list: WorkshopItemInfoResponseList = res.into_json_deserialize().unwrap();
        if verbose {
            println!("Response: {:#?}", list)
        }

        list.response.publishedfiledetails.pop().unwrap()
    }
}

pub mod steamworkshopdownloader_io {
    use anyhow::Result;
    use serde::Deserialize;

    const BASE_URL: &str = "https://api.steamworkshopdownloader.io/api/";

    #[derive(Debug, Deserialize)]
    pub struct DownloadRequestResponse {
        pub(crate) uuid: String,
    }

    pub fn request_transfer(item_id: u32, verbose: bool) -> Result<DownloadRequestResponse> {
        let body = format!("{{\"publishedFileId\":{},\"collectionId\":null,\"extract\":false,\"hidden\":false,\"direct\":false,\"autodownload\":true}}", item_id);
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
