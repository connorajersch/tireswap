import { FormEvent, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ApiError, SearchLocation, searchLocations } from "../api/client";

type LocationResult = SearchLocation & { id: string };

type SearchState = "idle" | "loading" | "success" | "no-results" | "error";

const formatLocationTitle = (location: SearchLocation): string => {
  const labels = [location.city, location.province].filter(
    (value): value is string => Boolean(value && value.trim().length > 0)
  );
  if (labels.length > 0) {
    return labels.join(", ");
  }

  if (location.postal_code) {
    return location.postal_code;
  }

  return `${location.lat.toFixed(4)}, ${location.lon.toFixed(4)}`;
};

const formatLocationMeta = (location: SearchLocation): string => {
  const labels = [];
  if (location.postal_code) {
    labels.push(location.postal_code);
  }
  labels.push(`${location.lat.toFixed(5)}, ${location.lon.toFixed(5)}`);
  return labels.join(" · ");
};

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

const Home = () => {
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const [status, setStatus] = useState<SearchState>("idle");
  const [results, setResults] = useState<LocationResult[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState("");

  const selectedLocation = useMemo(
    () => results.find((location) => location.id === selectedId) ?? null,
    [results, selectedId]
  );

  const executeSearch = async () => {
    const nextQuery = query.trim();
    if (!nextQuery) {
      setStatus("idle");
      setResults([]);
      setSelectedId(null);
      setErrorMessage("");
      return;
    }

    setStatus("loading");
    setErrorMessage("");
    setResults([]);
    setSelectedId(null);

    try {
      const response = await searchLocations(nextQuery);
      const nextResults: LocationResult[] = response.results.map((result, index) => ({
        ...result,
        id: `${result.lat}:${result.lon}:${index}`,
      }));

      if (nextResults.length === 0) {
        setStatus("no-results");
        return;
      }

      const [firstResult] = nextResults;
      setResults(nextResults);
      setSelectedId(firstResult?.id ?? null);
      setStatus("success");
    } catch (error) {
      if (error instanceof ApiError && error.status === 404) {
        setStatus("no-results");
        return;
      }

      const fallbackMessage = "Something went wrong while searching.";
      const nextErrorMessage = error instanceof ApiError
        ? readApiErrorMessage(error)
        : error instanceof Error
          ? error.message
          : fallbackMessage;
      setErrorMessage(nextErrorMessage);
      setStatus("error");
    }
  };

  const handleSearch = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    await executeSearch();
  };

  const handleContinue = () => {
    if (!selectedLocation) {
      return;
    }

    const locationLabel = formatLocationTitle(selectedLocation);
    const params = new URLSearchParams({
      location: locationLabel,
      latitude: selectedLocation.lat.toString(),
      longitude: selectedLocation.lon.toString(),
    });

    if (selectedLocation.postal_code) {
      params.set("postalCode", selectedLocation.postal_code);
    }

    navigate(`/results?${params.toString()}`);
  };

  return (
    <section className="page">
      <h1>Find your tire swap window</h1>
      <p className="muted">
        Search by city or postal code to get personalized recommendations.
      </p>

      <form className="location-search" onSubmit={handleSearch}>
        <label className="search-label" htmlFor="location-query">
          City or postal code
        </label>
        <div className="search-row">
          <input
            id="location-query"
            type="text"
            autoComplete="postal-code"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Try Toronto or M5V 2T6"
            className="search-input"
          />
          <button className="search-button" type="submit" disabled={status === "loading"}>
            {status === "loading" ? "Searching..." : "Search"}
          </button>
        </div>
        <p className="search-hint muted">
          Uses Google Geocoding to resolve your location to latitude/longitude.
        </p>
      </form>

      {status === "loading" && (
        <div className="status-card" aria-live="polite">
          Searching for matching locations...
        </div>
      )}

      {status === "no-results" && (
        <div className="status-card" role="status">
          No locations matched <strong>{query.trim()}</strong>. Try a nearby city or a full
          postal code.
        </div>
      )}

      {status === "error" && (
        <div className="status-card status-card-error" role="alert">
          <p>{errorMessage}</p>
          <button className="retry-button" type="button" onClick={() => void executeSearch()}>
            Retry
          </button>
        </div>
      )}

      {status === "success" && (
        <section className="selection-panel" aria-live="polite">
          <h2>Select your location</h2>
          <p className="muted">Choose the closest match before continuing.</p>
          <ul className="search-results">
            {results.map((location) => {
              const isSelected = location.id === selectedId;
              return (
                <li key={location.id}>
                  <button
                    type="button"
                    className={`result-option${isSelected ? " selected" : ""}`}
                    onClick={() => setSelectedId(location.id)}
                  >
                    <span className="result-title">{formatLocationTitle(location)}</span>
                    <span className="result-meta">{formatLocationMeta(location)}</span>
                  </button>
                </li>
              );
            })}
          </ul>
          <button
            type="button"
            className="continue-button"
            onClick={handleContinue}
            disabled={!selectedLocation}
          >
            Use selected location
          </button>
        </section>
      )}
    </section>
  );
};

export default Home;
