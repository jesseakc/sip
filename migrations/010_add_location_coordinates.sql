-- Typed coordinates for sites.
--
-- `locations.geo` was a free-form JSONB blob: no type checking, no index, and no
-- query ever selected it, so every Location came back from the API with
-- geo_latitude/geo_longitude hardcoded to None even though the seed data carried
-- {"lat": ..., "lng": ...}. A field technician's "which site am I at" needs a real
-- query, so promote the two values to typed columns and keep `geo` untouched for
-- anything already reading it.
--
-- PostGIS stays disabled (it is commented out in 001 and not required): a
-- bounding-box range scan on a B-tree index plus a haversine ordering is accurate
-- enough for site proximity and adds no extension dependency.

ALTER TABLE locations
    ADD COLUMN latitude  DOUBLE PRECISION,
    ADD COLUMN longitude DOUBLE PRECISION;

-- Backfill from the existing blob, but only when both values are genuinely
-- numeric — a bad string would abort the whole migration otherwise.
UPDATE locations
SET latitude  = (geo ->> 'lat')::double precision,
    longitude = (geo ->> 'lng')::double precision
WHERE latitude IS NULL
  AND geo ? 'lat'
  AND geo ? 'lng'
  AND (geo ->> 'lat') ~ '^-?[0-9]+(\.[0-9]+)?$'
  AND (geo ->> 'lng') ~ '^-?[0-9]+(\.[0-9]+)?$';

-- Reject coordinates that cannot be real, rather than storing them and returning
-- nonsense distances.
ALTER TABLE locations
    ADD CONSTRAINT locations_latitude_range
        CHECK (latitude IS NULL OR (latitude >= -90 AND latitude <= 90)),
    ADD CONSTRAINT locations_longitude_range
        CHECK (longitude IS NULL OR (longitude >= -180 AND longitude <= 180));

-- Proximity search reads a lat/lon bounding box first, then orders by distance.
-- Partial so rows without coordinates do not bloat the index.
CREATE INDEX idx_locations_coordinates
    ON locations (organization_id, latitude, longitude)
    WHERE latitude IS NOT NULL AND longitude IS NOT NULL;
