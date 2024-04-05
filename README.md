# Time Tracking Web Application

## Project Description
This project aims to create a simple yet effective time tracking web application tailored for freelancers, consultants, and small teams. The application allows users to manage clients, projects, and task categories, record time entries, and generate summaries of time spent and associated costs.

## Technologies
The following technologies are being considered for building the Time Tracking application:

### Frontend
* **React:** A popular JavaScript library for building user interfaces. Well-suited for creating a dynamic and interactive time-tracking interface.
* **Next.js (optional):** A framework built on top of React that provides features like server-side rendering, routing, and optimizations, streamlining development.

### Backend
* **Node.js:** A JavaScript runtime environment that allows JavaScript to be excuted outside the browser. Perfect for building the server-side logic of the application.
* **Express.js:** A web framework for Node.js that simplifies creating APIs and handling HTTP requests.

### Database
* **MariaDB/MySQL:** Popular relational databases for structured data. 
* **(Optional) Sequelize or TypeORM:** If using Node.js, these Object-Relational Mappers (ORMs) help manage database interactions.

### Additional Considerations
* **Authentication:** Google authentication integration for seamless user login.

**Note:** These are initial ideas. As the project evolves, other technologies might be a better fit.

## Entity-Relationship Diagram (ERD) Structure

### Entities and Attributes

#### Users
- `userID` (PK)
- `googleID`

#### Clients
- `clientID` (PK)
- `name`

#### Projects
- `projectID` (PK)
- `name`
- `description`
- `clientID` (FK)

#### Categories
- `categoryID` (PK)
- `name`
- `hourlyRate`

#### TimeEntries
- `timeEntryID` (PK)
- `date`
- `startTime`
- `endTime`
- `description`
- `projectID` (FK)
- `categoryID` (FK)

#### Expenses
- `expenseID` (PK)
- `amount`
- `description`
- `date`
- `projectID` (FK)

### Relationships

- **Users to Clients**: One-to-many (A user can have multiple clients).
- **Clients to Projects**: One-to-many (A client can have multiple projects).
- **Projects to TimeEntries**: One-to-many (A project has multiple time entries).
- **Projects to Expenses**: One-to-many (A project can have multiple expenses).
- **Categories to TimeEntries**: One-to-many (A time entry belongs to one category).

