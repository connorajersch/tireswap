# Tireswap - Project Plan

## Idea

Create a web service that tells users the optimal date that they should switch out their summer/winter tires based on available historical weather data.

## Theory

The government of Canada has made historical weather data freely available through an API. By analyzing this data, we can determine the average date of the first and last snowfall, as well as when temperatures are typically above or below 7 degrees celcius in a given area over the past several years. This information can help users decide when to switch their tires to ensure safety and compliance with local regulations.

## Technical Notes

### Backend

- Use rust for data aggregation and processing, as well as hosting the API server.
- Use a SQLite to store the processed data for quick retrieval.