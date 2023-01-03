# NOTE

To run in docker: fulfill the .env file with something like in .env.exmpl

# Image gallery

This is a small image gallery for the web. You can upload images, search and display them and private uploads are supported as well.

## Dependencies

The project requires Image Magick and the LibXML2 XML parser. On a Debian/Ubuntu system, you can install them by running `apt -y install libxml2-utils imagemagick`.

## Building

You need a working Rust compiler. Just run `cargo build` to build the debug version of the project or `cargo build --release` to get the release version.

## Database

You need a PostgreSQL database for the project. You can set the `DATABASE_URL` environment variable to an URL pointing to such a database, for example `postgres://postgres:rocket@127.0.0.1/postgres`, which is also the default value, should the variable not be set.

You can create such a database with the command `docker run --name some-postgres -e POSTGRES_PASSWORD=rocket  -p 5432:5432 -d postgres`.

## Running the project

You can start the application using `cargo run` or `cargo run --release`.

## docker-compose

You can also compile and run the project by running `docker-compose up`. Docker-compose will take care of the database as well.
