#Timetrack

## Time Tracking Tool ERD

This ERD represents the data model for a time-tracking application.

### Entities

#### Users
* userID (int) - PK
* googleID (string)

#### Clients
* clientID (int) - PK
* name (string)

#### Projects
* projectID (int) - PK
* name (string)
* description (text)
* clientID (int) - FK

#### Categories
* categoryID (int) - PK
* name (string)
* hourlyRate (decimal) 

#### TimeEntries
* timeEntryID (int) - PK
* date (date)
* startTime (time)
* endTime (time)
* description (text)
* projectID (int) - FK
* categoryID (int) - FK

#### Expenses
* expenseID (int) - PK
* amount (decimal)
* description (text)
* date (date)
* projectID (int) - FK

### Relationships

* Users - One-to-many (1:N) - Clients
* Clients - One-to-many (1:N) - Projects
* Projects - One-to-many (1:N) - TimeEntries
* Projects - One-to-many (1:N) - Expenses
* Categories - One-to-many (1:N) - TimeEntries

