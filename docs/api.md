# Internal API Endpoints

Lady's backend communicates with the frontend via TCP to `127.0.0.1`. All calls
are made from the frontend, and the backend provides JSON responses implemented
via the `Broker` trait.

Note that all API requests begin with `/api/v{versionId}` to seperate themselves
from user-facing pages.

### v1 API

| API | HTTP Method | Notes |
| :-- | :---------: | :---- |
| `database/tree` | GET | A tree of all databases, including their tables. Excludes data. |
| `database/tree/{dbname}` | GET | As above, but only for a single database. |
| `table/{dbName}` | GET | List of all tables in a single database |
