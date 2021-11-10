.headers on

WITH
  block_identity AS (
    SELECT
      json_extract(args, '$[0]') AS fractal_id,
      number AS block_number
    FROM extrinsics
    JOIN blocks ON hash=block
    WHERE section = "fractalMinting"
      AND method = "registerIdentity"
  ),
  first_registration AS (
    SELECT
      fractal_id,
      MIN(block_number) AS block_number
    FROM block_identity
    GROUP BY fractal_id
  ),
  registrations_on_week AS (
    SELECT
      (block_number / 14400 + 2) / 7 AS week,
      COUNT(*) AS count
    FROM first_registration
    GROUP BY 1
  ),
  aggregate_weekly AS (
    SELECT
      week,
      count AS increase,
      SUM(count) OVER (ROWS UNBOUNDED PRECEDING) AS total
    FROM registrations_on_week
    GROUP BY 1
  )
SELECT
  week,
  increase,
  total,
  100.0 * increase / LAG(total) OVER (ORDER BY week) AS percent_increase
FROM aggregate_weekly;
