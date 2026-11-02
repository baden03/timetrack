
# Time Tracking Application
## Technology-Agnostic Functional & Product Specification

**Version:** 1.0  
**Original development:** April–May 2024  
**Specification reconstructed:** November 2026  
**Status:** Existing prototype with planned authentication and deployment

---

# 1. Executive Summary

The Time Tracking Application is a project-based work-time management system intended primarily for freelancers, independent consultants, and small professional service businesses.

The application enables users to organize clients, manage projects, define billing categories, and record time spent working on individual projects.

Its central design principle is to make time recording and editing exceptionally simple, with minimal navigation and data entry overhead.

The primary workflow is:

1. Select a project from a visual dashboard.
2. Record time spent working on that project.
3. Review previous entries organized by date.
4. Modify existing entries directly within the table.
5. Automatically calculate working hours and daily totals.

The application also provides administrative interfaces for managing clients, projects, and categories.

The original implementation was a browser-based application backed by a relational database and an HTTP API. However, this specification deliberately avoids prescribing any particular implementation technology.

## 1.1 Primary Objectives

- Minimize the effort required to record working time.
- Organize work by client and project.
- Support multiple billing categories and hourly rates.
- Provide a clear overview of recorded work.
- Allow inline editing without navigating to separate forms.
- Maintain accurate time calculations.
- Provide a responsive, intuitive user interface.
- Support light and dark appearance modes.
- Protect application data through user authentication.
- Establish a foundation for future reporting and billing functionality.

## 1.2 Intended Users

The original application was designed around an individual professional managing work for multiple clients.

The architecture should support expansion to multiple authorized users, although the precise multi-user business rules were not finalized in the original development.

Typical users include:

- Freelance developers
- Consultants
- Designers
- Technical contractors
- Small professional service organizations

---

# 2. Application Architecture

## 2.1 Logical Components

The system consists of five logical layers.

### Presentation Layer

Provides the user interface for:

- Dashboard
- Client management
- Project management
- Category management
- Time tracking
- Authentication
- User administration
- Application preferences

### Application Logic Layer

Responsible for:

- Form validation
- Business rules
- Time calculations
- Data formatting
- State synchronization
- User interaction handling

### Service Layer

Exposes operations for:

- Client management
- Project management
- Category management
- Time entry management
- User management
- Authentication and authorization

### Persistence Layer

Stores:

- Clients
- Projects
- Categories
- Time entries
- Users
- Authentication-related information

### Security Layer

Responsible for:

- Identity verification
- Session management
- Access control
- Data protection

The implementation may use a monolithic application, a separate frontend and backend, or another suitable architecture.

---

# 3. Core Data Model

## 3.1 Client

A client represents an organization or individual for whom work is performed.

### Fields

| Field | Description |
|---|---|
| clientID | Unique client identifier |
| name | Client name |
| Additional fields | Contact and administrative information, as required |

The complete set of client contact fields was not established in the available development history.

### Relationships

- One client may have multiple projects.
- Each project belongs to one client.

### Operations

- Create client
- View clients
- Edit client
- Delete client

### Deletion Behavior

Deleting a client must also remove:

1. All projects belonging to that client.
2. All time entries associated with those projects.

This is a cascading deletion operation.

The application must request confirmation before destructive actions.

The implementation should ensure that related deletions are atomic, preventing partially deleted data.

---

## 3.2 Project

A project represents a distinct body of work performed for a client.

### Fields

| Field | Description |
|---|---|
| projectID | Unique project identifier |
| clientID | Reference to owning client |
| name | Project name |
| description | Project description |
| date | Project date field used in the original listing |

The original project list included a Date column. Its precise business meaning was not fully specified.

### Relationships

- Each project belongs to one client.
- A project may contain zero or more time entries.

### Operations

- Create project
- View projects
- Edit project
- Delete project
- Open project time tracking

### Project Creation

The project creation form must:

- Allow entry of a project name.
- Allow entry of a description.
- Present existing clients in a selection list.
- Store the selected client's identifier.
- Support required fields.
- Validate inputs before submission.
- Display a success message following creation.

### Project Editing

The project editing interface must:

- Load the existing project.
- Populate the form with stored values.
- Allow modification of project information.
- Allow reassignment to another client, where permitted.
- Save changes.
- Display appropriate feedback.

### Project Deletion

Deleting a project must also remove all associated time entries.

The deletion workflow must:

1. Request confirmation.
2. Execute the deletion.
3. Remove dependent time entries.
4. Return the user to the dashboard.
5. Display appropriate error feedback if deletion fails.

---

## 3.3 Category

Categories classify recorded work and may define billing rates.

Examples might include development, consulting, support, and administration.

These examples are illustrative; no fixed category list was established.

### Fields

| Field | Description |
|---|---|
| categoryID | Unique category identifier |
| name | Category name |
| hourlyRate | Hourly billing rate |

### Hourly Rate

- Must support decimal values.
- Example: 123.45
- Must preserve appropriate monetary precision.
- Currency was not explicitly fixed in the original specification.

### Operations

- Create category
- View categories
- Edit category
- Delete category

### Relationships

Each time entry references a category.

The original backend included a fallback to categoryID 1 when no category was provided, although the frontend required users to select a category.

For a new implementation, category selection should be explicit and consistently validated.

The behavior for deleting categories that are referenced by existing time entries requires a product decision.

---

## 3.4 Time Entry

A time entry represents a period of work performed on a project.

### Fields

| Field | Description |
|---|---|
| timeEntryID | Unique identifier |
| projectID | Associated project |
| categoryID | Associated work category |
| date | Date work was performed |
| startTime | Beginning of work interval |
| endTime | End of work interval |
| hours | Calculated duration |
| description | Description of work performed |

### Relationships

- Each time entry belongs to one project.
- Each time entry references one category.
- Multiple entries may exist for the same date.
- A project may have no time entries.

### Time Representation

Time is entered and displayed using a 24-hour clock.

Example:

Start: 08:30  
End: 12:45  
Duration: 4.25 hours

Time values may be stored with seconds internally.

Example:

08:30:00

However, the user interface displays:

08:30

The original design explored restricting input to 15-minute increments.

This should be treated as a configurable or explicitly enforced validation rule rather than relying solely on the native time input control.

---

# 4. Dashboard

## 4.1 Purpose

The dashboard is the primary entry point for authenticated users.

Its main purpose is to provide quick access to active projects and their time-tracking interfaces.

## 4.2 Project Card Grid

Projects are displayed as a grid of cards.

Each card contains:

**Title:** Project name  
**Subtitle:** Client name  
**Body:** Project description

### Interaction

The entire project card is clickable.

Selecting a card opens the time-tracking interface for that project.

### Requirements

- Cards must be visually distinguishable.
- Cards must support keyboard and pointer navigation.
- The layout must adapt to available screen width.
- Cards should be usable on desktop and mobile devices.
- Long descriptions should be handled gracefully.

## 4.3 Empty State

When no projects exist, the dashboard should display a suitable message and provide a clear path to create the first project.

---

# 5. Navigation

The application uses a persistent navigation structure.

## 5.1 Primary Navigation

### Dashboard

Returns to the project card grid.

### Projects

Opens the complete project listing.

Sub-navigation:

- Add New Project

### Clients

Opens client management.

Sub-navigation:

- Add New Client

### Categories

Provides access to category management.

The exact placement of category management within the original navigation was not fully documented.

### User / Account

Planned functionality:

- User profile
- Sign out
- User administration, where authorized

## 5.2 Navigation Requirements

- Navigation should not unnecessarily reload the entire application.
- The current location should be identifiable.
- Navigation must be available throughout authenticated sections.
- Dropdown menus must be accessible using keyboard controls.

---

# 6. Project Management Interface

## 6.1 Project List

In addition to the dashboard card grid, the application provides a traditional project table.

The original table included:

- Project ID
- Project name
- Client
- Date
- Description

### Interaction

Clicking the project name opens the project editing interface.

The same project-listing functionality may be reused in different contexts, with configurable link destinations.

For example:

- Project administration → Edit Project
- Time management → Track Time

## 6.2 Create and Edit Forms

Forms must support:

- Required fields
- Input validation
- Selection controls
- Error messages
- Success messages
- Submission to persistent storage

The create and edit workflows should share validation rules and reusable field definitions wherever practical.

---

# 7. Time Tracking Interface

## 7.1 Purpose

The time-tracking view is the central operational component of the application.

Each project has its own time-tracking page.

The page contains two main sections:

1. New time entry form
2. Existing time entries grouped by date

The project name appears prominently at the top.

Example:

TimeTrack: Website Development

---

# 8. New Time Entry Form

## 8.1 Layout

The original interface uses a compact, single-line form.

Fields are arranged horizontally where sufficient screen width is available.

### Fields

| Field | Control |
|---|---|
| Start time | Time picker |
| End time | Time picker |
| Category | Selection list |
| Description | Multiline text |
| Submit | Action button |

The project identifier is determined by the currently selected project and does not need to be entered manually.

The work date is required by the underlying data model, but the original form did not expose a date input.

The original controller allowed the date to be omitted, although its fallback handling was incomplete.

A new implementation should explicitly define the default work date, preferably the current local date, and decide whether users may change it.

## 8.2 Validation

Required fields:

- Start time
- End time
- Category
- Description

Validation must occur before submission.

Invalid fields should display understandable messages.

Examples:

- Start time is required.
- End time is required.
- Category is required.
- A description is required.

The application should also validate time intervals.

The original design did not fully settle whether work periods crossing midnight are supported.

## 8.3 Submission Behavior

When a valid entry is submitted:

1. Validate the data.
2. Create the entry.
3. Calculate its duration.
4. Persist the entry.
5. Clear the input fields.
6. Refresh the displayed time entries.
7. Update affected daily totals.

The application must perform these operations without a full-page reload.

## 8.4 Keyboard Behavior

The description field supports:

**Enter:** Submit the form.

**Shift + Enter:** Insert a new line into the description.

Programmatic submission should use the application's normal form submission mechanism so that validation and submission logic remain centralized.

## 8.5 Feedback

The user should receive feedback when:

- An entry is successfully created.
- Validation fails.
- The server rejects the entry.
- A network or persistence error occurs.

---

# 9. Time Entry Listing

## 9.1 Date-Based Grouping

Time entries are grouped by their work date.

Each date receives a separate table.

Example:

### 23.04.2024 — 6.5h

| From | To | Hrs. | Cat. | Description |
|---|---|---|---|---|
| 08:00 | 10:30 | 2.5 | Development | API work |
| 11:00 | 15:00 | 4 | Support | Client requests |

### 22.04.2024 — 3.75h

| From | To | Hrs. | Cat. | Description |
|---|---|---|---|---|
| 09:00 | 12:45 | 3.75 | Development | Bug fixes |

## 9.2 Date Formatting

Dates are displayed in German format:

DD.MM.YYYY

Example:

23.04.2024

Dates should be grouped using a consistent calendar-date representation to avoid timezone-related grouping errors.

## 9.3 Table Columns

The original table used the following column configuration:

| Data Field | Display Name | Editing Control |
|---|---|---|
| startTime | From | Time input |
| endTime | To | Time input |
| hours | Hrs. | Calculated / read-only |
| catName | Cat. | Category selection |
| description | Description | Textarea |

The presentation order and labels are controlled by a column configuration.

This allows developers to:

- Reorder columns
- Rename column headings
- Hide fields
- Define editing controls
- Mark fields as read-only

The column configuration should not depend on the order of fields returned by the database.

## 9.4 Hidden Fields

Internal fields such as:

- timeEntryID
- projectID
- categoryID
- date

are available to the application but are not necessarily displayed as table columns.

They remain available for identifying records, editing relationships, and performing updates.

## 9.5 Empty State

Projects with no recorded work must display a meaningful empty state.

Example:

"No time entries have been recorded for this project."

An empty result is a valid state, not an application error.

---

# 10. Inline Editing

## 10.1 Objective

Users should be able to edit existing time entries directly within the time-entry table.

The intended experience resembles inline database-table editing tools.

The user should not need to navigate to a separate edit form.

## 10.2 Activation

Double-clicking an editable table cell activates editing mode.

The displayed value is replaced with an appropriate input control.

Only the selected cell enters editing mode.

## 10.3 Input Types

### Start Time

Time picker.

### End Time

Time picker.

### Category

Dropdown containing available categories.

The displayed category name is mapped to its stored category identifier.

### Description

Multiline textarea.

### Hours

Read-only calculated value.

Hours should not be directly editable unless a future business requirement explicitly permits manual duration overrides.

## 10.4 Editing State

The application tracks:

- The time entry being edited
- The field being edited
- The original value
- The current edited value
- The save status

The editing state must be isolated from unrelated rows.

## 10.5 Save on Blur

When the user moves focus away from the editing control:

1. Validate the edited value.
2. Determine whether the value changed.
3. Save the modified field.
4. Update the local application state.
5. Return the cell to display mode.

If the value has not changed, the application should simply exit editing mode without making an unnecessary update request.

## 10.6 Keyboard Interaction

For textareas:

- Enter saves the value.
- Shift + Enter inserts a newline.

For single-line controls:

- Enter saves the value.

Recommended additional behavior:

- Escape cancels editing.
- Tab commits the current value and advances focus.

These additional behaviors were not confirmed in the original implementation but are appropriate for the intended interaction model.

## 10.7 Save Failures

If saving fails:

- Do not silently discard the edited value.
- Display an error.
- Allow retry or cancellation.
- Preserve the last successfully saved database value.

## 10.8 Data Synchronization

The application should avoid maintaining unnecessary duplicate copies of the same data.

A suitable logical model is:

- Maintain one authoritative collection of time entries in application state.
- Derive date groups from that collection.
- Maintain temporary editing values separately.
- Update the authoritative collection after successful persistence.

This prevents synchronization problems between grouped entries and the original data collection.

---

# 11. Time Calculation

## 11.1 Individual Entry Duration

The duration of a time entry is calculated from its start and end times.

Formula:

Duration in hours =
(End time - Start time) / 60 minutes

Example:

Start: 09:15  
End: 12:45

Duration:

3 hours 30 minutes = 3.5 hours

## 11.2 Decimal Representation

Hours are represented as decimal hours, not hours and minutes.

Examples:

| Time Duration | Decimal Hours |
|---|---|
| 15 minutes | 0.25 |
| 30 minutes | 0.5 |
| 45 minutes | 0.75 |
| 1 hour | 1 |
| 1 hour 15 minutes | 1.25 |
| 2 hours 30 minutes | 2.5 |

## 11.3 Formatting

Unnecessary trailing zeros are removed.

Examples:

12.00 → 12  
12.50 → 12.5  
12.75 → 12.75

## 11.4 Daily Totals

Each date group displays the sum of its entries.

Formula:

Daily Total =
Sum of all entry durations for that date

Example:

2.5 + 1.25 + 3 = 6.75h

## 11.5 Recalculation After Editing

If the start or end time of an entry changes:

1. Save the new time.
2. Recalculate the entry duration.
3. Persist the updated duration.
4. Update the entry in application state.
5. Recalculate the affected date's total.
6. Update the visible table and heading.

Example:

Original entry:

09:00–11:00 = 2h

Edited entry:

09:00–11:15 = 2.25h

The individual duration and daily total must update immediately after successful saving.

## 11.6 Source of Truth

The backend or central business logic should be responsible for authoritative duration calculations.

The frontend may calculate temporary previews, but stored values should remain consistent with the authoritative calculation rules.

## 11.7 Precision

Duration calculations should preferably use integer minutes internally.

For example:

135 minutes = 2.25 hours

This avoids unnecessary floating-point accumulation errors.

## 11.8 Overnight Work

The original prototype explored calculations that add 24 hours when the end time precedes the start time.

The precise business rule was not finalized.

A new implementation must decide whether to:

- Reject intervals crossing midnight.
- Permit overnight intervals.
- Split overnight intervals between calendar dates.

This decision affects date grouping and daily totals.

---

# 12. User Authentication

## 12.1 Status

Authentication was planned but had not been confirmed complete in the original project history.

The original plan called for Google account authentication using OAuth 2.0 / OpenID Connect.

The new implementation may select another secure identity provider or authentication mechanism.

## 12.2 Public Landing Page

Unauthenticated visitors should see a public landing page.

The page should:

- Introduce the application.
- Explain its purpose.
- Provide a login action.
- Prevent unauthorized access to private project data.

## 12.3 Login Page

The login page should provide:

- A clear sign-in action.
- Authentication status feedback.
- Error handling.
- Appropriate redirects after successful login.

## 12.4 External Identity Provider

The original plan was to support login with a Google account.

Required logical capabilities:

1. Redirect the user to the identity provider.
2. Authenticate the user.
3. Verify the returned identity.
4. Associate the identity with an application user.
5. Establish an authenticated session.
6. Redirect to the dashboard.

Authentication should use established security standards rather than custom password or token handling.

## 12.5 User Data

Suggested user fields:

| Field | Description |
|---|---|
| userID | Internal user identifier |
| email | Email address |
| displayName | User's display name |
| identityProvider | Authentication provider |
| externalSubjectID | Provider's stable user identifier |
| role | Application role |
| status | Active / disabled |
| createdAt | Account creation timestamp |
| lastLoginAt | Most recent login |

These fields are proposed for the new implementation; the original Users table was not yet established.

## 12.6 User Administration

Authorized administrators should be able to:

- View users
- Add or invite users
- Edit user details
- Assign roles
- Disable users
- Re-enable users
- Revoke sessions where supported

## 12.7 Authorization

The original application did not finalize its authorization model.

A minimum suggested model is:

### Administrator

- Manage users
- Manage clients
- Manage projects
- Manage categories
- Manage all permitted time entries

### Standard User

- Access assigned or permitted projects
- Record time
- Edit permitted time entries
- View permitted project information

Whether all users share a common workspace or have isolated data requires an explicit decision.

## 12.8 Protected Application Areas

Unauthenticated users must not be able to access private application data.

Security enforcement must occur at the service or backend layer, not only through hidden frontend navigation.

---

# 13. Appearance and User Experience

## 13.1 Light and Dark Themes

The application supports:

- Light mode
- Dark mode

A theme switch is available in the application header.

The original implementation used a slider-style toggle rather than a conventional button.

## 13.2 Theme Persistence

The selected theme should persist across:

- Page navigation
- Application reloads
- Subsequent sessions on the same device

## 13.3 Theme Variables

The visual design should use centralized theme tokens for:

- Background color
- Text color
- Navigation background
- Table backgrounds
- Alternating table rows
- Input backgrounds
- Borders
- Buttons
- Success messages
- Error messages
- Dropdown menus

Components should not rely on hard-coded colors that break when switching themes.

## 13.4 Responsive Layout

The application should support:

- Desktop browsers
- Tablets
- Mobile browsers

The single-line time entry form may wrap or use a stacked layout on narrow screens.

Tables should remain readable and usable without losing access to editing functionality.

## 13.5 Accessibility

The interface should support:

- Keyboard navigation
- Visible focus indicators
- Proper input labels
- Accessible error messages
- Sufficient color contrast
- Semantic table structure
- Accessible confirmation dialogs

Double-click editing should have an alternative activation mechanism for keyboard and touch users.

---

# 14. Validation and Error Handling

## 14.1 Client-Side Validation

Used for immediate feedback.

Examples:

- Missing required values
- Invalid time formats
- Missing category selection
- Invalid monetary values

## 14.2 Server-Side Validation

All business rules must also be enforced by the backend.

Client-side validation alone is insufficient.

## 14.3 Error Categories

The application should distinguish:

- Validation errors
- Authentication errors
- Authorization errors
- Missing resources
- Empty result sets
- Database errors
- Network errors
- Unexpected application errors

## 14.4 Empty Collections

A valid project with no time entries should return an empty collection.

It should not be treated as a missing project.

Similarly, empty project and client lists should produce usable empty-state interfaces.

## 14.5 Success Feedback

Successful create, update, and delete operations should provide appropriate feedback without unnecessarily interrupting the user's workflow.

---

# 15. Data Integrity

## 15.1 Referential Integrity

The data model must enforce relationships between:

- Clients and projects
- Projects and time entries
- Categories and time entries

## 15.2 Cascading Deletions

Required behavior:

Client deletion
  → Delete client's projects
  → Delete associated time entries

Project deletion
  → Delete associated time entries

These operations should use database transactions or equivalent atomic mechanisms.

## 15.3 Confirmation

Destructive actions require explicit confirmation.

The original project used confirmation dialogs before deleting records.

## 15.4 Concurrent Updates

For a multi-user implementation, the developer should establish a strategy to prevent one user's changes from silently overwriting another user's edits.

Possible approaches include record versioning or optimistic concurrency control.

This was not part of the confirmed original prototype.

---

# 16. Service Operations

The implementation should expose logical operations corresponding to the application's functional requirements.

The transport mechanism is not prescribed.

## 16.1 Clients

- List clients
- Get client
- Create client
- Update client
- Delete client

## 16.2 Projects

- List projects
- Get project
- List projects by client
- Create project
- Update project
- Delete project

## 16.3 Categories

- List categories
- Get category
- Create category
- Update category
- Delete category

## 16.4 Time Entries

- List entries for a project
- Get time entry
- Create time entry
- Update time entry
- Delete time entry

An update operation must support modifying individual fields without overwriting unrelated values.

## 16.5 Users

- List authorized users
- Get user
- Create or invite user
- Update user
- Disable user
- Manage user roles

## 16.6 Authentication

- Start authentication
- Process authentication result
- Get current session
- Sign out
- Verify permissions

---

# 17. Reporting and Billing

## 17.1 Original Scope

The original application recorded:

- Work durations
- Projects
- Clients
- Categories
- Category hourly rates

These provide a foundation for reporting and billing.

However, a complete reporting or invoicing module was not confirmed as implemented.

## 17.2 Potential Reporting Features

Future development may include:

- Total hours by project
- Total hours by client
- Hours by category
- Monthly work summaries
- Date-range filtering
- Billable value calculations
- Export of time entries
- Invoice preparation

## 17.3 Billing Calculation

A possible billing calculation is:

Entry Value =
Recorded Hours × Applicable Hourly Rate

A future billing implementation must decide whether the applicable rate is:

- The category's current rate
- A project-specific rate
- A historical rate captured when work was recorded

Historical rate preservation is preferable for reliable financial reporting.

These billing rules remain future requirements rather than confirmed original functionality.

---

# 18. Expenses

The original project planning mentioned an Expenses entity and future expense-related API operations.

No complete expense management workflow was established in the available development history.

Possible future capabilities include:

- Record project expenses
- Associate expenses with clients and projects
- Categorize expenses
- Record amounts and currencies
- Attach receipts
- Mark expenses as billable
- Include expenses in reporting

Expense functionality should remain a separate optional module until its requirements are defined.

---

# 19. Non-Functional Requirements

## 19.1 Performance

- Navigation should be responsive.
- Inline editing should not require page reloads.
- Time-entry creation should update the visible list efficiently.
- Date grouping and totals should update automatically.
- Large time-entry collections should remain manageable.

## 19.2 Reliability

- Failed saves must not silently lose user input.
- Database operations must preserve consistency.
- Destructive operations must be atomic.
- The system should recover gracefully from network failures.

## 19.3 Security

- Authenticated access to private data
- Server-side authorization
- Secure session handling
- Encrypted communication
- Protection against injection attacks
- Protection against cross-site scripting
- Protection against cross-site request forgery where applicable
- Secure handling of credentials and secrets

## 19.4 Maintainability

- Separate business rules from presentation.
- Centralize validation.
- Centralize time calculation logic.
- Use reusable form and table components.
- Keep field configuration declarative.
- Avoid duplicated application state.
- Maintain clear service contracts.

## 19.5 Data Protection

The implementation should define:

- Backup procedures
- Retention policies
- Data export capabilities
- Account deletion behavior
- Access logging where appropriate

These requirements were not fully specified in the original prototype.

---

# 20. Testing Requirements

## 20.1 Client Tests

Verify:

- Client creation
- Client editing
- Client deletion
- Cascading deletion of projects and entries

## 20.2 Project Tests

Verify:

- Project creation
- Project editing
- Client selection
- Project deletion
- Dashboard card navigation
- Empty project handling

## 20.3 Time Entry Tests

Verify:

- Entry creation
- Required-field validation
- Category selection
- Correct project association
- Date grouping
- Duration calculation
- Daily total calculation
- Decimal formatting
- German date formatting

## 20.4 Inline Editing Tests

Verify:

- Double-click activation
- Correct input control
- Editing existing values
- Save on blur
- Enter to save
- Shift + Enter for newline
- Cancellation behavior
- Save error handling
- Local state synchronization

## 20.5 Calculation Tests

Verify:

- 15-minute intervals
- Decimal-hour conversion
- Start-time changes
- End-time changes
- Recalculation of individual hours
- Recalculation of daily totals
- Midnight boundary behavior once specified

## 20.6 Authentication Tests

Verify:

- Login
- Logout
- Session expiration
- Unauthorized access rejection
- Protected navigation
- User permissions
- Disabled accounts

## 20.7 Appearance Tests

Verify:

- Light mode
- Dark mode
- Theme persistence
- Responsive layouts
- Accessible navigation

---

# 21. Original Project Status

The following status reflects the last explicit project progress update in May 2024 and the available development discussions.

## 21.1 Backend

Reported complete:

- Database schema and relationships
- Client management operations
- Project management operations
- Category management operations
- Time-entry management operations
- Inline editing support
- Time calculation and adjustment logic

Still open:

- User authentication

Expense management remained a future consideration despite the broad backend completion report.

## 21.2 Frontend

Implemented or substantially developed:

- Project dashboard
- Project cards
- Project list
- Project creation and editing
- Client management interfaces
- Category management interfaces
- Time entry form
- Date-grouped time-entry tables
- Inline editing
- Light and dark themes
- Application navigation

The complete final state of every frontend feature was not independently verified.

## 21.3 Integration

Reported complete:

- Frontend/backend communication
- Basic error handling and validation

Ongoing:

- Responsiveness and interactivity optimization
- Testing and debugging

## 21.4 Authentication

Planned:

- Users table
- User management
- Public landing page
- Login page
- Google authentication
- Session management
- Protected routes
- Authorization

No completion of these tasks was confirmed.

## 21.5 Deployment

Production deployment was planned to follow authentication.

No completed production deployment was confirmed in the available history.

---

# 22. Recommended Implementation Phases

A developer rebuilding this application may use the following phased approach.

## Phase 1 — Core Data and Administration

Deliver:

- Data model
- Client CRUD
- Project CRUD
- Category CRUD
- Referential integrity
- Validation
- Destructive-action confirmation

Acceptance criterion:

Users can manage clients, projects, and categories reliably.

## Phase 2 — Dashboard and Time Recording

Deliver:

- Project card dashboard
- Project navigation
- Time entry creation
- Date-based grouping
- Time calculation
- Daily totals

Acceptance criterion:

Users can select a project, record work, and review accurate daily totals.

## Phase 3 — Inline Editing

Deliver:

- Editable time and description cells
- Category selection
- Save on blur
- Keyboard interactions
- Error recovery
- Reactive recalculation

Acceptance criterion:

Users can modify recorded work without navigating away or refreshing the page.

## Phase 4 — Authentication and User Management

Deliver:

- Public landing page
- Login flow
- Identity provider integration
- User administration
- Sessions
- Protected application areas
- Authorization

Acceptance criterion:

Only authorized users can access and modify permitted data.

## Phase 5 — Production Readiness

Deliver:

- Responsive design refinement
- Accessibility review
- Automated tests
- Security review
- Deployment configuration
- HTTPS
- Backups
- Monitoring

Acceptance criterion:

The application is reliable and suitable for regular production use.

## Phase 6 — Optional Business Features

Potential enhancements:

- Project and client reporting
- Billable-hour summaries
- Billing-rate history
- Invoice generation
- Expense tracking
- Data exports
- Advanced search and filtering

These features should be scoped separately.

---

# 23. Open Product Decisions

Before beginning a new implementation, the developer and product owner should resolve the following questions.

1. Is the application intended for one user or multiple users?

2. If multiple users are supported, are clients and projects shared or isolated?

3. What permissions should standard users and administrators have?

4. Should Google remain the preferred identity provider?

5. Should the system support additional authentication methods?

6. Should users be able to edit the work date?

7. Should the default work date always be today?

8. Are 15-minute increments mandatory or merely preferred?

9. Can time entries cross midnight?

10. Should overlapping time entries be allowed?

11. Can time entries have zero duration?

12. Should category hourly rates apply globally or vary by project?

13. Should historical billing rates be preserved?

14. What should happen when a category with existing time entries is deleted?

15. Should project and client deletion remain permanent, or should soft deletion be introduced?

16. Should users be able to undo a deletion?

17. What reporting and export capabilities are required?

18. Is invoicing within scope?

19. Is expense management within scope?

20. What hosting, backup, privacy, and data-retention requirements apply?

---

# 24. Product Acceptance Criteria

The initial application can be considered functionally complete when:

- Authorized users can log in and log out.
- Users can create and manage clients.
- Users can create and manage projects.
- Users can define work categories and hourly rates.
- The dashboard displays projects as clickable cards.
- Selecting a project opens its time-tracking interface.
- Users can record time with minimal data entry.
- Time entries are grouped by date.
- Individual durations are calculated correctly.
- Daily totals are displayed correctly.
- Users can edit entries directly within the table.
- Time changes automatically update duration and totals.
- Changes persist without requiring a page refresh.
- Invalid data is rejected with useful feedback.
- Deleting a project removes its dependent time entries.
- Deleting a client removes its projects and dependent time entries.
- The interface supports light and dark themes.
- The application works across common screen sizes.
- Private data is protected through authentication and authorization.
- The application can be deployed and operated reliably.

---

# 25. Guiding Design Philosophy

The defining characteristic of this application is its low-friction time-recording workflow.

A user should be able to:

1. Open the application.
2. Select the relevant project.
3. Enter a time range, category, and description.
4. Submit the entry.
5. Immediately see the updated record and daily total.

If a correction is needed, the user should be able to double-click the relevant value, modify it, and continue working.

Administrative features should remain accessible without dominating the main interface.

The product should prioritize:

**Speed, simplicity, accuracy, and minimal interruption.**

Technology choices should support these goals rather than dictate the user experience.

---