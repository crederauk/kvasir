/*******************************************************************************
 Chinook Database - Version 1.4
 Script: Chinook_MySql.sql
 Description: Creates and populates the Chinook database.
 DB Server: MySql
 Author: Luis Rocha
 License: http://www.codeplex.com/ChinookDatabase/license
 ********************************************************************************/
/*******************************************************************************
 Create Tables
 ********************************************************************************/
CREATE TABLE `Album` (
    `AlbumId` INT NOT NULL,
    `Title` NVARCHAR(160) NOT NULL,
    `ArtistId` INT NOT NULL,
    CONSTRAINT `PK_Album` PRIMARY KEY (`AlbumId`)
);
CREATE TABLE `Artist` (
    `ArtistId` INT NOT NULL,
    `Name` NVARCHAR(120),
    CONSTRAINT `PK_Artist` PRIMARY KEY (`ArtistId`)
);
CREATE TABLE `Customer` (
    `CustomerId` INT NOT NULL,
    `FirstName` NVARCHAR(40) NOT NULL,
    `LastName` NVARCHAR(20) NOT NULL,
    `Company` NVARCHAR(80),
    `Address` NVARCHAR(70),
    `City` NVARCHAR(40),
    `State` NVARCHAR(40),
    `Country` NVARCHAR(40),
    `PostalCode` NVARCHAR(10),
    `Phone` NVARCHAR(24),
    `Fax` NVARCHAR(24),
    `Email` NVARCHAR(60) NOT NULL,
    `SupportRepId` INT,
    CONSTRAINT `PK_Customer` PRIMARY KEY (`CustomerId`)
);
CREATE TABLE `Employee` (
    `EmployeeId` INT NOT NULL,
    `LastName` NVARCHAR(20) NOT NULL,
    `FirstName` NVARCHAR(20) NOT NULL,
    `Title` NVARCHAR(30),
    `ReportsTo` INT,
    `BirthDate` DATETIME,
    `HireDate` DATETIME,
    `Address` NVARCHAR(70),
    `City` NVARCHAR(40),
    `State` NVARCHAR(40),
    `Country` NVARCHAR(40),
    `PostalCode` NVARCHAR(10),
    `Phone` NVARCHAR(24),
    `Fax` NVARCHAR(24),
    `Email` NVARCHAR(60),
    CONSTRAINT `PK_Employee` PRIMARY KEY (`EmployeeId`)
);
CREATE TABLE `Genre` (
    `GenreId` INT NOT NULL,
    `Name` NVARCHAR(120),
    CONSTRAINT `PK_Genre` PRIMARY KEY (`GenreId`)
);
CREATE TABLE `Invoice` (
    `InvoiceId` INT NOT NULL,
    `CustomerId` INT NOT NULL,
    `InvoiceDate` DATETIME NOT NULL,
    `BillingAddress` NVARCHAR(70),
    `BillingCity` NVARCHAR(40),
    `BillingState` NVARCHAR(40),
    `BillingCountry` NVARCHAR(40),
    `BillingPostalCode` NVARCHAR(10),
    `Total` NUMERIC(10, 2) NOT NULL,
    CONSTRAINT `PK_Invoice` PRIMARY KEY (`InvoiceId`)
);
CREATE TABLE `InvoiceLine` (
    `InvoiceLineId` INT NOT NULL,
    `InvoiceId` INT NOT NULL,
    `TrackId` INT NOT NULL,
    `UnitPrice` NUMERIC(10, 2) NOT NULL,
    `Quantity` INT NOT NULL,
    CONSTRAINT `PK_InvoiceLine` PRIMARY KEY (`InvoiceLineId`)
);
CREATE TABLE `MediaType` (
    `MediaTypeId` INT NOT NULL,
    `Name` NVARCHAR(120),
    CONSTRAINT `PK_MediaType` PRIMARY KEY (`MediaTypeId`)
);
CREATE TABLE `Playlist` (
    `PlaylistId` INT NOT NULL,
    `Name` NVARCHAR(120),
    CONSTRAINT `PK_Playlist` PRIMARY KEY (`PlaylistId`)
);
CREATE TABLE `PlaylistTrack` (
    `PlaylistId` INT NOT NULL,
    `TrackId` INT NOT NULL,
    CONSTRAINT `PK_PlaylistTrack` PRIMARY KEY (`PlaylistId`, `TrackId`)
);
CREATE TABLE `Track` (
    `TrackId` INT NOT NULL,
    `Name` NVARCHAR(200) NOT NULL,
    `AlbumId` INT,
    `MediaTypeId` INT NOT NULL,
    `GenreId` INT,
    `Composer` NVARCHAR(220),
    `Milliseconds` INT NOT NULL,
    `Bytes` INT,
    `UnitPrice` NUMERIC(10, 2) NOT NULL,
    CONSTRAINT `PK_Track` PRIMARY KEY (`TrackId`)
);