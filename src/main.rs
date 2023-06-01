use async_zip::tokio::write::ZipFileWriter as AsyncZipFileWriter;
use async_zip::{Compression, ZipEntryBuilder};
use mimalloc::MiMalloc;
use poem::{listener::TcpListener, web::Multipart, Result, Route, Server};
use poem_openapi::{
    param::Path,
    payload::{Attachment, AttachmentType, Json},
    ApiResponse, Object, OpenApi, OpenApiService,
};
use std::ops::Deref;
use std::path::Path as FilePath;
use std::path::PathBuf;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const IMAGE_TYPE: [&str; 5] = ["jpg", "jpeg", "png", "gif", "webp"];

#[derive(Debug, Object, Clone)]
struct ResultVo {
    code: u16,
    msg: String,
    data: Option<String>,
}

#[derive(ApiResponse)]
enum UploadResponse {
    #[oai(status = 200)]
    Success(Json<ResultVo>),
    #[oai(status = 500)]
    InternalServerError(Json<ResultVo>),
}

#[derive(ApiResponse)]
enum DownloadResponse {
    #[oai(status = 200)]
    Success(Attachment<Vec<u8>>),
    #[oai(status = 500)]
    InternalServerError(Json<ResultVo>),
}

#[derive(ApiResponse)]
enum DeleteResponse {
    #[oai(status = 200)]
    Success(Json<ResultVo>),
}

async fn zip_files(paths: &Vec<String>) -> tokio::io::Result<Vec<u8>> {
    let mut file = Vec::new();
    let mut writer = AsyncZipFileWriter::with_tokio(&mut file);
    for path in paths {
        let path_buf = PathBuf::from(&path);
        let file_name = path_buf.file_name().unwrap().to_str().unwrap();
        let data = tokio::fs::read(&path).await?;
        let builder = ZipEntryBuilder::new(file_name.into(), Compression::Deflate);
        writer
            .write_entry_whole(builder, &data)
            .await
            .map_err(|e| {
                zip::result::ZipError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("无法写入文件: {}", e),
                ))
            })?;
    }
    writer.close().await.map_err(|e| {
        zip::result::ZipError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("无法关闭文件: {}", e),
        ))
    })?;
    Ok(file)
}

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/upload/:dir_type/:dir_id", method = "post")]
    async fn upload(
        &self,
        dir_type: Path<String>,
        dir_id: Path<u64>,
        mut multipart: Multipart,
    ) -> Result<UploadResponse> {
        let dir_path = format!("/root/image/{}/{}", dir_type.deref(), dir_id.deref());
        if !FilePath::new(&dir_path).exists() {
            tokio::fs::create_dir_all(&dir_path).await.map_err(|e| {
                UploadResponse::InternalServerError(Json(ResultVo {
                    code: 500,
                    msg: format!("在创建目录时出错: {}", e),
                    data: None,
                }))
            })?;
        }
        let mut files: Vec<(String, Vec<u8>)> = Vec::new();
        let mut multipart_count = 0;
        while let Some(field) = multipart.next_field().await? {
            let file_name = field.file_name().map(|s| s.to_owned()).unwrap();
            let data = field.bytes().await.map_err(|e| {
                UploadResponse::InternalServerError(Json(ResultVo {
                    code: 500,
                    msg: format!("读取上传的文件失败: {}", e),
                    data: None,
                }))
            })?;
            if !IMAGE_TYPE.contains(&file_name.split(".").last().unwrap()) {
                return Err(UploadResponse::InternalServerError(Json(ResultVo {
                    code: 500,
                    msg: "上传的文件不是图片".to_string(),
                    data: None,
                }))
                .into());
            }
            if data.len() > 200 * 1024 {
                return Err(UploadResponse::InternalServerError(Json(ResultVo {
                    code: 500,
                    msg: "文件大小超过200kb".to_string(),
                    data: None,
                }))
                .into());
            }
            files.push((file_name, data));
            multipart_count += 1;
            if multipart_count > 4 {
                return Err(UploadResponse::InternalServerError(Json(ResultVo {
                    code: 500,
                    msg: "上传的文件数量超过4个".to_string(),
                    data: None,
                }))
                .into());
            }
        }
        let mut file_read_dir = tokio::fs::read_dir(&dir_path).await.map_err(|e| {
            UploadResponse::InternalServerError(Json(ResultVo {
                code: 500,
                msg: format!("无法打开指定的目录: {}", e),
                data: None,
            }))
        })?;
        let mut file_count: u8 = 0;
        while let Some(_) = file_read_dir.next_entry().await.map_err(|e| {
            UploadResponse::InternalServerError(Json(ResultVo {
                code: 500,
                msg: format!("无法读取目录: {}", e),
                data: None,
            }))
        })? {
            file_count += 1;
        }
        if file_count + multipart_count > 4 {
            return Err(UploadResponse::InternalServerError(Json(ResultVo {
                code: 500,
                msg: "上传的文件数量超过4个".to_string(),
                data: None,
            }))
            .into());
        }
        for (file_name, data) in files {
            tokio::fs::write(format!("{}/{}", dir_path, file_name), data)
                .await
                .map_err(|e| {
                    UploadResponse::InternalServerError(Json(ResultVo {
                        code: 500,
                        msg: format!("写入文件失败: {}", e),
                        data: None,
                    }))
                })?;
        }

        Ok(UploadResponse::Success(Json(ResultVo {
            code: 200,
            msg: "图片上传成功".to_string(),
            data: None,
        })))
    }

    #[oai(path = "/download/:dir_type/:dir_id", method = "get")]
    async fn download(
        &self,
        dir_type: Path<String>,
        dir_id: Path<u64>,
    ) -> Result<DownloadResponse> {
        let dir_path = format!("/root/image/{}/{}", dir_type.deref(), dir_id.deref());
        let mut paths = match tokio::fs::read_dir(&dir_path).await {
            Ok(paths) => paths,
            Err(_) => {
                tokio::fs::create_dir_all(&dir_path).await.map_err(|e| {
                    DownloadResponse::InternalServerError(Json(ResultVo {
                        code: 500,
                        msg: format!("在创建目录时出错: {}", e),
                        data: None,
                    }))
                })?;
                tokio::fs::read_dir(&dir_path).await.map_err(|e| {
                    DownloadResponse::InternalServerError(Json(ResultVo {
                        code: 500,
                        msg: format!("无法打开指定的目录: {}", e),
                        data: None,
                    }))
                })?
            }
        };
        let mut paths_vec = Vec::new();
        while let Some(path) = paths.next_entry().await.map_err(|e| {
            DownloadResponse::InternalServerError(Json(ResultVo {
                code: 500,
                msg: format!("无法读取目录: {}", e),
                data: None,
            }))
        })? {
            let path_str = path.file_name().into_string().unwrap();
            paths_vec.push(format!("{}/{}", &dir_path, &path_str));
        }
        let zip_file = zip_files(&paths_vec).await.map_err(|e| {
            DownloadResponse::InternalServerError(Json(ResultVo {
                code: 500,
                msg: format!("压缩文件时出错: {}", e),
                data: None,
            }))
        })?;
        Ok(DownloadResponse::Success(
            Attachment::new(zip_file)
                .filename("archive.zip".to_string())
                .attachment_type(AttachmentType::Attachment),
        ))
    }

    #[oai(path = "/download/:dir_type/:dir_id/:file_name", method = "get")]
    async fn download_file(
        &self,
        dir_type: Path<String>,
        dir_id: Path<u64>,
        file_name: Path<String>,
    ) -> Result<DownloadResponse> {
        let dir_path = format!("/root/image/{}/{}", dir_type.deref(), dir_id.deref());
        let file_path = format!("{}/{}", dir_path, file_name.to_string());
        let path_vec = vec![file_path];
        let zip_file = zip_files(&path_vec).await.map_err(|e| {
            DownloadResponse::InternalServerError(Json(ResultVo {
                code: 500,
                msg: format!("压缩文件时出错: {}", e),
                data: None,
            }))
        })?;
        Ok(DownloadResponse::Success(
            Attachment::new(zip_file)
                .filename(file_name.to_string())
                .attachment_type(AttachmentType::Attachment),
        ))
    }

    #[oai(path = "/delete/:dir_type/:dir_id", method = "get")]
    async fn delete(&self, dir_type: Path<String>, dir_id: Path<u64>) -> Result<DeleteResponse> {
        let dir_path = format!("/root/image/{}/{}", dir_type.deref(), dir_id.deref());
        tokio::fs::remove_dir_all(&dir_path).await.map_err(|e| {
            DeleteResponse::Success(Json(ResultVo {
                code: 500,
                msg: format!("目录删除失败: {}", e),
                data: None,
            }))
        })?;
        Ok(DeleteResponse::Success(Json(ResultVo {
            code: 200,
            msg: "目录删除成功".to_string(),
            data: None,
        })))
    }

    #[oai(path = "/delete/:dir_type/:dir_id/:file_name", method = "get")]
    async fn delete_file(
        &self,
        dir_type: Path<String>,
        dir_id: Path<u64>,
        file_name: Path<String>,
    ) -> Result<DeleteResponse> {
        let dir_path = format!("/root/image/{}/{}", dir_type.deref(), dir_id.deref());
        let file_path = format!("{}/{}", dir_path, file_name.deref());
        tokio::fs::remove_file(&file_path).await.map_err(|e| {
            DeleteResponse::Success(Json(ResultVo {
                code: 500,
                msg: format!("文件删除失败: {}", e),
                data: None,
            }))
        })?;
        Ok(DeleteResponse::Success(Json(ResultVo {
            code: 200,
            msg: "文件删除成功".to_string(),
            data: None,
        })))
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let api_service = OpenApiService::new(Api, "Image host", "1.0").server("http://localhost:8082");
    let app = Route::new().nest("/", api_service);

    Server::new(TcpListener::bind("0.0.0.0:8082"))
        .run(app)
        .await
}
