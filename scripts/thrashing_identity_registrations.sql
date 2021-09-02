WITH
  registrations AS (
    SELECT
      json_extract(args, '$[0]') AS fractal_id,
      json_extract(args, '$[1]') AS substrate_address,
      number AS block_number
    FROM extrinsics
    JOIN blocks ON hash=block
    WHERE section = "fractalMinting"
      AND method = "registerIdentity"
  ),
  thrash_count AS (
    SELECT COUNT(*) as thrashes,
      fractal_id,
      substrate_address
    FROM registrations
    GROUP BY fractal_id, substrate_address
  ),
  problematic AS (
    SELECT fractal_id, SUM(thrashes) AS thrashes
    FROM thrash_count
    GROUP BY fractal_id
    HAVING SUM(thrashes) >= 6
  )
SELECT COUNT(*)
FROM problematic;
