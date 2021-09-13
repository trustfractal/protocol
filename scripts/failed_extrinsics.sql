WITH
  failed_extrinsics AS (
    SELECT
      number,
      block_index
    FROM extrinsics
    JOIN blocks ON hash=block
    WHERE NOT success
  )
SELECT
  number / (14400) AS period,
  COUNT(*)
FROM failed_extrinsics
GROUP BY 1
ORDER BY 1 DESC;
