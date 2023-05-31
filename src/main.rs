// import io
// import os
// import shutil
// import zipfile
// import aiofiles
// from typing import List

// import uvicorn
// from fastapi import FastAPI, File, UploadFile, Response, Request
// from fastapi.responses import JSONResponse

// class ImageTooManyException(Exception):
//     def __init__(self, name: str):
//         self.name = name

// class DirNotExistException(Exception):
//     def __init__(self, name: str):
//         self.name = name

// class ImageNotExistException(Exception):
//     def __init__(self, name: str):
//         self.name = name

// class NonImageException(Exception):
//     def __init__(self, name: str):
//         self.name = name

// app = FastAPI()

// image_suffix = ['jpg', 'png', 'jpeg', 'bmp', 'gif', 'webp']

// @app.exception_handler(ImageTooManyException)
// async def unicorn_exception_handler(request: Request, exc: ImageTooManyException):
//     return JSONResponse(
//         status_code=500,
//         content={
//             'code': 1019,
//             'msg': '上传图片超过4张，请删除后再上传',
//             'data': exc.name
//         }
//     )

// @app.exception_handler(DirNotExistException)
// async def unicorn_exception_handler(request: Request, exc: DirNotExistException):
//     return JSONResponse(
//         status_code=500,
//         content={
//             'code': 1020,
//             'msg': '文件夹不存在',
//             'data': exc.name
//         }
//     )

// @app.exception_handler(ImageNotExistException)
// async def unicorn_exception_handler(request: Request, exc: ImageNotExistException):
//     return JSONResponse(
//         status_code=500,
//         content={
//             'code': 1021,
//             'msg': '图片不存在',
//             'data': exc.name
//         }
//     )

// @app.exception_handler(NonImageException)
// async def unicorn_exception_handler(request: Request, exc: NonImageException):
//     return JSONResponse(
//         status_code=500,
//         content={
//             'code': 1022,
//             'msg': '上传的文件中包含非图片文件',
//             'data': exc.name
//         }
//     )

// def check_if_files_too_many(path):
//     length = len(os.listdir(path))
//     if length >= 4:
//         raise ImageTooManyException(name='image more than 4 error')

// def check_if_files_are_image(files):
//     for file in files:
//         if file.filename.split('.')[-1] not in image_suffix:
//             raise ImageTooManyException(name='some files are not image')

// async def zip_files(filenames):
//     s = io.BytesIO()
//     zf = zipfile.ZipFile(s, 'w')
//     for fpath in filenames:
//         if fpath.split('.')[-1] not in image_suffix:
//             continue
//         _, fname = os.path.split(fpath)
//         async with aiofiles.open(fpath, 'rb') as f:
//             zf.writestr(fname, await f.read())
//     zf.close()
//     return Response(
//         s.getvalue(),
//         media_type='application/x-zip-compressed',
//         headers={
//             'Content-Disposition': 'attachment;filename=archive.zip'
//         }
//     )

// @ app.post('/upload/{_type}/{_id}')
// async def upload_image(_type: str, _id: int, files: List[UploadFile] = File(...)):
//     # check_if_files_are_image(files)
//     if not os.path.exists(f'/root/image/{_type}/{_id}'):
//         os.makedirs(f'/root/image/{_type}/{_id}')
//     if (len(files) + len(os.listdir(f'/root/image/{_type}/{_id}'))) > 4:
//         raise ImageTooManyException(name='image more than 4 error')
//     if files[0].filename == '':
//         return {
//             'code': 500,
//             'msg': 'no image uploaded',
//             'data': [f'/root/image/{_type}/{_id}/' + i for i in os.listdir(f'/root/image/{_type}/{_id}') if i.split('.')[-1] in image_suffix]
//         }
//     try:
//         for file in files:
//             check_if_files_too_many(f'/root/image/{_type}/{_id}')
//             if file.filename.split('.')[-1] not in image_suffix:
//                 continue
//             async with aiofiles.open(f'/root/image/{_type}/{_id}/{file.filename}', 'wb') as f:
//                 await f.write(await file.read())
//     except Exception as e:
//         return {
//             'code': 500,
//             'msg': 'image upload fail',
//             'data': [f'/root/image/{_type}/{_id}/' + i for i in os.listdir(f'/root/image/{_type}/{_id}') if i.split('.')[-1] in image_suffix]
//         }
//     return {
//         'code': 200,
//         'msg': 'image upload success',
//         'data': [f'/root/image/{_type}/{_id}/' + i for i in os.listdir(f'/root/image/{_type}/{_id}') if i.split('.')[-1] in image_suffix]
//     }

// @ app.get('/download/{_type}/{_id}')
// async def download_image(_type: str, _id: int):
//     if not os.path.exists(f'/root/image/{_type}/{_id}'):
//         raise DirNotExistException(name='dir not exist')
//     filenames = [f'/root/image/{_type}/{_id}/' +
//                  i for i in os.listdir(f'/root/image/{_type}/{_id}')]
//     return await zip_files(filenames)

// @ app.get('/download/{_type}/{_id}/{filename}')
// async def download_image(_type: str, _id: int, filename: str):
//     if not os.path.exists(f'/root/image/{_type}/{_id}'):
//         raise DirNotExistException(name='dir not exist')
//     return await zip_files([f'/root/image/{_type}/{_id}/{filename}'])

// @ app.get('/delete/{_type}/{_id}/{filename}')
// def delete_image(_type: str, _id: int, filename: str):
//     if not os.path.exists(f'/root/image/{_type}/{_id}'):
//         raise DirNotExistException(name='dir not exist')
//     if not os.path.exists(f'/root/image/{_type}/{_id}/{filename}'):
//         raise ImageNotExistException(name='image not exist')
//     os.remove(f'/root/image/{_type}/{_id}/{filename}')
//     # 如果文件夹已经空了，就删除这个文件夹
//     if len(os.listdir(f'/root/image/{_type}/{_id}')) == 0:
//         shutil.rmtree(f'/root/image/{_type}/{_id}')
//     return {
//         'code': 200,
//         'msg': 'image delete success',
//         'data': None
//     }

// @ app.get('/delete/{_type}/{_id}')
// def delete_image(_type: str, _id: int):
//     if not os.path.exists(f'/root/image/{_type}/{_id}'):
//         raise DirNotExistException(name='dir not exist')
//     shutil.rmtree(f'/root/image/{_type}/{_id}')
//     return {
//         'code': 200,
//         'msg': 'image delete success',
//         'data': None
//     }

// if __name__ == '__main__':
//     uvicorn.run(app, host='0.0.0.0', port=8082)

use std::collections::HashMap;

use poem::{
    error::{BadRequest, InternalServerError},
    listener::TcpListener,
    Result, Route, Server,
};
use poem_openapi::{
    param::Path,
    payload::{Attachment, AttachmentType, Json},
    types::multipart::Upload,
    ApiResponse, Multipart, Object, OpenApi, OpenApiService,
};
use std::path::Path as FilePath;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::Mutex;
use zip::write::FileOptions;
use zip::ZipWriter;

#[derive(Debug, Object, Clone)]
struct File {
    name: String,
    desc: Option<String>,
    content_type: Option<String>,
    filename: Option<String>,
    data: Vec<u8>,
}

#[derive(Debug, ApiResponse)]
enum FileResponse {
    #[oai(status = 200)]
    Ok(Attachment<Vec<u8>>),
    /// File not found
    #[oai(status = 404)]
    NotFound,
}

struct Status {
    id: u64,
    files: HashMap<u64, File>,
}

#[derive(Debug, Multipart)]
struct UploadPayload {
    name: String,
    desc: Option<String>,
    file: Vec<Upload>,
}

#[derive(Debug, Object, Clone)]
struct ResultVo {
    code: u16,
    msg: String,
    data: Option<String>,
}

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/upload", method = "post")]
    async fn upload(
        &self,
        dir_type: Path<String>,
        dir_id: Path<u64>,
        upload: UploadPayload,
    ) -> Result<Json<ResultVo>> {
        let dir_path = format!(
            "/root/image/{}/{}",
            dir_type.to_string(),
            dir_id.to_string()
        );
        if !FilePath::new(&dir_path).exists() {
            tokio::fs::create_dir_all(&dir_path)
                .await
                .map_err(InternalServerError)?;
        }
        for file in upload.file {
            let filename = file.file_name().map(|s| s.to_owned());
            let data = file.into_vec().await.map_err(BadRequest);
            tokio::fs::write(format!("{}/{}", dir_path, filename.unwrap()), data.unwrap())
                .await
                .unwrap();
        }
        Ok(Json(ResultVo {
            code: 200,
            msg: "image upload success".to_string(),
            data: None,
        }))
    }

    // /// Get file
    // #[oai(path = "/files/:id", method = "get")]
    // async fn get(&self, id: Path<u64>) -> GetFileResponse {
    //     let status = self.status.lock().await;
    //     match status.files.get(&id) {
    //         Some(file) => {
    //             let mut attachment =
    //                 Attachment::new(file.data.clone()).attachment_type(AttachmentType::Attachment);
    //             if let Some(filename) = &file.filename {
    //                 attachment = attachment.filename(filename);
    //             }
    //             GetFileResponse::Ok(attachment)
    //         }
    //         None => GetFileResponse::NotFound,
    //     }
    // }

    async fn zip_files(paths: &[&str]) -> zip::result::ZipResult<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for path in paths {
            let path_buf = PathBuf::from(path);
            let file_name = path_buf.file_name().unwrap().to_str().unwrap();
            let mut file = tokio::fs::File::open(path).await?;
            zip.start_file(file_name, options)?;
            // 我怎么把这些文件都写进zip里面去呢？
            file.read_to_end(&mut buffer).await?;
            zip.write_all(&*buffer).await?;
        }

        zip.finish()?;
        Ok(zip.into_inner()?)
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let api_service =
        OpenApiService::new(Api {}, "Upload Files", "1.0").server("http://localhost:3000/api");

    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(Route::new().nest("/api", api_service))
        .await
}
