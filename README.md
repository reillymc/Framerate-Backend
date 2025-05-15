# Framerate Backend

Framerate is a movie and TV watch journal that tracks ratings and viewing habits over time. With this data, Framerate helps users identify trends in their viewing preferences and refine their opinions.

## Features

-   **Movie and TV Show Support**: Track movies and TV shows.
-   **Watch History**: Record and store watches.
-   **Review Recording and History**: Write reviews and track ratings for watches.
-   **Watchlisting**: Create a list of shows or movies to watch.
-   **Upcoming and Popular Media**: View upcoming releases and popular titles.

## Development

The provided dev container sets up a pre-configured development environment, including Diesel CLI, environment variables, and testing databases. To use the dev container, export environment variables from the `.devcontainer/.env` file (as shown in `.devcontainer/.env.example`).

To run the server and watch for changes, use: `cargo watch -x run`.

### Architecture

Framerate is built using the [Actix-Web](https://actix.rs/) framework and [Diesel](https://diesel.rs/).

### Database

The PostgreSQL database is incrementally built with [Diesel migrations](https://diesel.rs/guides/getting-started.html#setup-diesel-for-your-project) defined in the [migrations](./migrations/) directory.

### Tools

A [Bruno](https://www.usebruno.com/) collection is included for manual API testing and exploration.

## Testing

To set up or update the test database, run: `./tools/Scripts/setup_test_db.sh`. This is required when implementing new migrations to keep the dev database up-to-date.

Run Framerate's test suite with: `cargo pretty-test` or `cargo test`.

## Deployment

The included [Dockerfile](./Dockerfile) and [compose](./compose.yml) file can be used to deploy Framerate as a containerized service. Alternatively, you can build and run Framerate as a binary, setting up a database separately. For both cases, refer to the [env example file](./.env.example) for required environment variables.

**Setup Procedure:**

1. Send a POST request to the Setup endpoint with the secret key from the `SETUP_SECRET` environment variable.
2. Receive a token with administrator permissions and use it to send a POST request to the User Create endpoint with user details in the body.
3. Create at least one admin user, as the setup endpoint will no longer provide admin tokens once a user is present in the database.

## Contributing

I created Framerate as a way to learn some of the basics of Rust and also because I felt existing tools / services did not cater to the 'historical' journaling of watches and ratings. There are many features I intend to add and parts I aim to improve, but further work is not guaranteed. Pull requests are welcome if they are within the the projects current scope and / or goals, otherwise raise an issue and I would be happy to discuss problems or suggestions.
