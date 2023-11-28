use actix_web::HttpResponse;

pub(crate) async fn dev_docs_index_handler() -> impl actix_web::Responder {
    let body = r#"
    <!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8">
    <title>Swagger UI</title>
    <link rel="stylesheet" type="text/css" href="/assets/swagger/dist/swagger-ui.css" />
    <link rel="stylesheet" type="text/css" href="/assets/swagger/dist/index.css" />
    <link rel="icon" type="image/png" href="/assets/swagger/dist/favicon-32x32.png" sizes="32x32" />
    <link rel="icon" type="image/png" href="/assets/swagger/dist/favicon-16x16.png" sizes="16x16" />
  </head>

  <body>
    <div id="swagger-ui"></div>
    <script src="/assets/swagger/dist/swagger-ui-bundle.js" charset="UTF-8"> </script>
    <script src="/assets/swagger/dist/swagger-ui-standalone-preset.js" charset="UTF-8"> </script>
    <script src="/assets/swagger/entry.js" charset="UTF-8"> </script>
  </body>
</html>
    "#;
    HttpResponse::Ok()
        .insert_header(actix_web::http::header::ContentType::html())
        .body(body)
}
