use gzlib::proto::{
  sku_image::SkuObj,
  sku_image::{
    sku_image_server::*, CoverBulkRequest, CoverObj, NewImageId, NewRequest, SkuRequest,
  },
};
use packman::VecPack;
use sku_image_microservice::{
  image::{self, SkuImageExt},
  prelude::*,
};
use std::{env, path::Path};
use std::{error::Error, path::PathBuf};
use tokio::{fs::read_dir, process::Command, sync::Mutex};
use tokio::{fs::remove_file, prelude::*};
use tokio::{
  fs::{create_dir_all, File},
  sync::oneshot,
};
use tonic::{transport::Server, Request, Response, Status};

use gzlib::proto;

struct SkuImageService {
  skus: Mutex<VecPack<image::SkuImage>>,
}

impl SkuImageService {
  pub fn init(skus: VecPack<image::SkuImage>) -> Self {
    Self {
      skus: Mutex::new(skus),
    }
  }

  async fn add_new(&self, r: NewRequest) -> ServiceResult<String> {
    // If we have SKU already created
    if let Ok(sku) = self.skus.lock().await.find_id_mut(&r.sku) {
      let res = sku
        .as_mut()
        .unpack()
        .add_image(r.file_name, r.file_extension, r.image_bytes)
        .map_err(|e| ServiceError::bad_request(&e))?;
      return Ok(res);
    }

    // Otherwise create sku
    let mut new_sku = image::SkuImage::new(r.sku);
    // and add image
    let res = new_sku
      .add_image(r.file_name, r.file_extension, r.image_bytes)
      .map_err(|e| ServiceError::bad_request(&e))?;
    // Finally add new_sku object to skus db
    self
      .skus
      .lock()
      .await
      .insert(new_sku)
      .map_err(|e| ServiceError::bad_request(&e.to_string()))?;

    Ok(res)
  }

  async fn get_images(&self, r: SkuRequest) -> ServiceResult<SkuObj> {
    let res = self.skus.lock().await.find_id(&r.sku)?.unpack().clone();
    Ok(res.into())
  }

  async fn get_cover_bulk(&self, r: CoverBulkRequest) -> ServiceResult<Vec<CoverObj>> {
    let mut res: Vec<CoverObj> = Vec::new();
    for sku in self
      .skus
      .lock()
      .await
      .as_vec_mut()
      .iter_mut()
      .filter(|s| r.sku_ids.contains(&s.unpack().sku))
    {
      let _sku = sku.unpack();
      if let Some(cover) = _sku.get_cover() {
        res.push(CoverObj {
          sku: _sku.sku,
          cover_image_id: cover,
        });
      }
    }
    Ok(res)
  }
}

#[tonic::async_trait]
impl SkuImage for SkuImageService {
  async fn add_new(
    &self,
    request: Request<proto::sku_image::NewRequest>,
  ) -> Result<Response<proto::sku_image::NewImageId>, Status> {
    let res = self.add_new(request.into_inner()).await?;
    Ok(Response::new(NewImageId { new_image_id: res }))
  }

  async fn set_cover(
    &self,
    request: Request<proto::sku_image::SetCoverRequest>,
  ) -> Result<Response<proto::sku_image::SkuObj>, Status> {
    todo!()
  }

  async fn swap_images(
    &self,
    request: Request<proto::sku_image::SwapRequest>,
  ) -> Result<Response<proto::sku_image::SkuObj>, Status> {
    todo!()
  }

  async fn remove(
    &self,
    request: Request<proto::sku_image::RemoveRequest>,
  ) -> Result<Response<proto::sku_image::SkuObj>, Status> {
    todo!()
  }

  async fn get_images(
    &self,
    request: Request<proto::sku_image::SkuRequest>,
  ) -> Result<Response<proto::sku_image::SkuObj>, Status> {
    let res = self.get_images(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  type GetCoverBulkStream = tokio::sync::mpsc::Receiver<Result<CoverObj, Status>>;

  async fn get_cover_bulk(
    &self,
    request: Request<proto::sku_image::CoverBulkRequest>,
  ) -> Result<Response<Self::GetCoverBulkStream>, Status> {
    // Create channel for stream response
    let (mut tx, rx) = tokio::sync::mpsc::channel(100);

    // Get resources as Vec<SourceObject>
    let res = self.get_cover_bulk(request.into_inner()).await?;

    // Send the result items through the channel
    tokio::spawn(async move {
      for ots in res.into_iter() {
        tx.send(Ok(ots)).await.unwrap();
      }
    });

    // Send back the receiver
    Ok(Response::new(rx))
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  // Init CARTS database
  let sku_images: VecPack<image::SkuImage> =
    VecPack::load_or_init(PathBuf::from("data/sku_images"))
      .expect("Error while loading sku_images db");

  let addr = env::var("SERVICE_ADDR_SKU_IMAGE")
    .unwrap_or("[::1]:50082".into())
    .parse()
    .unwrap();

  // Create shutdown channel
  let (tx, rx) = oneshot::channel();

  // Spawn the server into a runtime
  tokio::task::spawn(async move {
    Server::builder()
      .add_service(SkuImageServer::new(SkuImageService::init(sku_images)))
      .serve_with_shutdown(addr, async {
        let _ = rx.await;
      })
      .await
      .unwrap()
  });

  tokio::signal::ctrl_c().await?;

  println!("SIGINT");

  // Send shutdown signal after SIGINT received
  let _ = tx.send(());

  Ok(())
}
