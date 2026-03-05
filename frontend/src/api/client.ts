const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;
const API_TOKEN = import.meta.env.VITE_API_TOKEN;

if (!API_BASE_URL) {
  // eslint-disable-next-line no-console
  console.warn("VITE_API_BASE_URL is not set. API calls will fail.");
}

export class ApiError extends Error {
  status: number;
  details?: unknown;

  constructor(message: string, status: number, details?: unknown) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.details = details;
  }
}

type RequestOptions = Omit<RequestInit, "headers"> & {
  headers?: Record<string, string>;
};

export const apiClient = async <T>(
  path: string,
  options: RequestOptions = {}
): Promise<T> => {
  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...(API_TOKEN ? { Authorization: `Bearer ${API_TOKEN}` } : {}),
      ...options.headers,
    },
  });

  if (!response.ok) {
    let details: unknown = undefined;
    try {
      details = await response.json();
    } catch {
      details = await response.text();
    }
    throw new ApiError("API request failed", response.status, details);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return (await response.json()) as T;
};

export type SearchLocation = {
  city: string | null;
  province: string | null;
  postal_code: string | null;
  lat: number;
  lon: number;
  source: string;
};

export type SearchApiResponse = {
  query: string;
  normalized_query: string;
  results: SearchLocation[];
};

export const searchLocations = async (query: string): Promise<SearchApiResponse> => {
  const params = new URLSearchParams({ query });
  return apiClient<SearchApiResponse>(`/api/search?${params.toString()}`);
};

export type StationSummary = {
  id: number;
  name: string;
  distance_km: number;
};

export type DistanceSummary = {
  min: number | null;
  avg: number | null;
  max: number | null;
};

export type SeasonalQuality = {
  stations_with_data: number;
  coverage_pct: number;
};

export type DataYearsSummary = {
  min_span_years: number | null;
  avg_span_years: number | null;
  max_span_years: number | null;
};

export type OptimalDatesResponse = {
  latitude: number;
  longitude: number;
  switch_to_summer: string | null;
  switch_to_winter: string | null;
  stations_analyzed: number;
  stations: {
    requested: number;
    returned: number;
    list: StationSummary[];
    distance_km: DistanceSummary;
  };
  quality: {
    summer: SeasonalQuality;
    winter: SeasonalQuality;
    data_years: DataYearsSummary;
  };
};

export const getOptimalDates = async (
  latitude: number,
  longitude: number,
  numStations = 5
): Promise<OptimalDatesResponse> => {
  const params = new URLSearchParams({
    latitude: latitude.toString(),
    longitude: longitude.toString(),
    num_stations: numStations.toString(),
  });
  return apiClient<OptimalDatesResponse>(`/api/optimal-dates?${params.toString()}`);
};
