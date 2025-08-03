use std::path::PathBuf;
use qcos::client::Client;
use qcos::objects::{mime};
use qcos::request::ErrNo;

pub trait CosFunction {
    fn create_cos_client(secret_id: String, secret_key: String, region: String, bucket: String) -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;

    async fn upload_object(&self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>>;

    async fn download_object(&self, key: String, name: String) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(Debug, Clone)]
pub struct TecentCosUtil {
    client: Client,
    bucket: String
}

impl CosFunction for TecentCosUtil {
    fn create_cos_client(secret_id: String, secret_key: String, region: String, bucket: String) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::new(secret_id, secret_key, bucket.clone(), region);
        Ok(
            TecentCosUtil {client: client, bucket: bucket}
        )
    }

    async fn upload_object(&self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let object_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("default_name");
        let suffix = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        let mime_type = match suffix {
            "jpg" | "jpeg" => mime::IMAGE_JPEG,
            "png" => mime::IMAGE_PNG,
            "gif" => mime::IMAGE_GIF,
            _ => mime::APPLICATION_OCTET_STREAM, // Default to binary stream
        };
        //let mut acl_header = AclHeader::new();
        // acl_header.insert_object_x_cos_acl(ObjectAcl::BucketOwnerFullControl);
        println!("Uploading object: {} with MIME type: {}", object_name, mime_type);
        let res = self.client.put_object(&path, object_name, Some(mime_type), None).await;
        if res.error_no == ErrNo::SUCCESS {
            println!("success");
        } else {
            println!("[{}]: {} {:?}", res.error_no, res.error_message, String::from_utf8(res.result));
        }
        Ok(())
    }

    async fn download_object(&self, key: String, name: String) -> Result<(), Box<dyn std::error::Error>> {
        self.client.get_object(&key, &name, None).await;
        Ok(())
    }
}
