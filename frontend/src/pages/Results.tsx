import { useEffect, useMemo, useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { ApiError, OptimalDatesResponse, getOptimalDates } from "../api/client";

type ResultStatus = "idle" | "loading" | "success" | "error";

const readApiErrorMessage = (error: ApiError): string => {
  if (typeof error.details === "string" && error.details.trim().length > 0) {
    return error.details;
  }

  if (error.details && typeof error.details === "object") {
    const details = error.details as { error?: { message?: string } };
    const message = details.error?.message;
    if (message) {
      return message;
    }
  }

  return error.message;
};

const formatDistance = (value: number | null): string =>
  value === null ? "N/A" : `${value.toFixed(1)} km`;

const formatPercent = (value: number): string => `${value.toFixed(0)}%`;
const formatYears = (value: number | null): string =>
  value === null ? "N/A" : value.toFixed(1);

const Results = () => {
  const [searchParams] = useSearchParams();
  const [status, setStatus] = useState<ResultStatus>("idle");
  const [errorMessage, setErrorMessage] = useState("");
  const [data, setData] = useState<OptimalDatesResponse | null>(null);

  const locationLabel = searchParams.get("location") ?? "your location";
  const postalCode = searchParams.get("postalCode");
  const latitudeRaw = searchParams.get("latitude");
  const longitudeRaw = searchParams.get("longitude");

  const coordinates = useMemo(() => {
    if (!latitudeRaw || !longitudeRaw) {
      return null;
    }

    const latitude = Number(latitudeRaw);
    const longitude = Number(longitudeRaw);
    if (Number.isNaN(latitude) || Number.isNaN(longitude)) {
      return null;
    }

    return { latitude, longitude };
  }, [latitudeRaw, longitudeRaw]);

  useEffect(() => {
    const load = async () => {
      if (!coordinates) {
        setStatus("error");
        setErrorMessage("Missing coordinates. Please search for a location first.");
        setData(null);
        return;
      }

      setStatus("loading");
      setErrorMessage("");
      setData(null);

      try {
        const response = await getOptimalDates(
          coordinates.latitude,
          coordinates.longitude
        );
        setData(response);
        setStatus("success");
      } catch (error) {
        const fallbackMessage = "Failed to load recommendation results.";
        const nextError = error instanceof ApiError
          ? readApiErrorMessage(error)
          : error instanceof Error
            ? error.message
            : fallbackMessage;
        setErrorMessage(nextError);
        setStatus("error");
      }
    };

    void load();
  }, [coordinates]);

  return (
    <section className="page">
      <h1>Recommendation results</h1>
      <p className="muted">
        Based on historical climate data near <strong>{locationLabel}</strong>
        {postalCode ? ` (${postalCode})` : ""}.
      </p>

      {status === "loading" && (
        <div className="status-card" aria-live="polite">
          Calculating optimal switch dates...
        </div>
      )}

      {status === "error" && (
        <div className="status-card status-card-error" role="alert">
          <p>{errorMessage}</p>
          <Link className="back-link" to="/">
            Back to search
          </Link>
        </div>
      )}

      {status === "success" && data && (
        <div className="results-layout">
          <section className="result-hero">
            <div className="result-pill">
              <span className="result-pill-label">Switch to summer</span>
              <strong>{data.switch_to_summer ?? "No recommendation"}</strong>
            </div>
            <div className="result-pill">
              <span className="result-pill-label">Switch to winter</span>
              <strong>{data.switch_to_winter ?? "No recommendation"}</strong>
            </div>
          </section>

          <section className="result-card">
            <h2>Station coverage</h2>
            <div className="metrics-grid">
              <div>
                <span className="metric-label">Stations analyzed</span>
                <strong>{data.stations_analyzed}</strong>
              </div>
              <div>
                <span className="metric-label">Distance range</span>
                <strong>
                  {formatDistance(data.stations.distance_km.min)} -{" "}
                  {formatDistance(data.stations.distance_km.max)}
                </strong>
              </div>
              <div>
                <span className="metric-label">Summer coverage</span>
                <strong>{formatPercent(data.quality.summer.coverage_pct)}</strong>
              </div>
              <div>
                <span className="metric-label">Winter coverage</span>
                <strong>{formatPercent(data.quality.winter.coverage_pct)}</strong>
              </div>
              <div>
                <span className="metric-label">Data years (avg)</span>
                <strong>{formatYears(data.quality.data_years.avg_span_years)}</strong>
              </div>
            </div>
          </section>

          <section className="result-card">
            <h2>Nearest stations</h2>
            <ul className="station-list">
              {data.stations.list.map((station) => (
                <li key={station.id}>
                  <span>{station.name}</span>
                  <span>{formatDistance(station.distance_km)}</span>
                </li>
              ))}
            </ul>
            <div className="actions-row">
              <Link className="back-link" to="/">
                Search another location
              </Link>
            </div>
          </section>
        </div>
      )}
    </section>
  );
};

export default Results;
