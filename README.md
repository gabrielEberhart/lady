# Lady

Lady is a locally-hosted database management tool designed to make small-scope
testing databases accessible to the end user. It is great for educational demos,
mock database implementations in testing environments, and tech demonstrations.

## Project Structure

Lady is seperated into two projects: a webserver and a database manager. The
webserver runs in `127.0.0.1` and provides a Bootstrap interface for the end
user to communicate with the MariaDB manager. The webserver also hosts several
API endpoints which are used to send data to the database manager.
