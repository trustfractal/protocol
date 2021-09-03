WITH
  minting_registrations AS (
    SELECT
      number / 14400 AS period,
      signer
    FROM extrinsics
    JOIN blocks ON hash=block
    WHERE section = "fractalMinting"
      AND method = "registerForMinting"
      AND success
  ),
  minting_rewards AS (
    SELECT
      COUNT(DISTINCT period) * 3 AS total_reward,
      signer
    FROM minting_registrations
    WHERE
      period >= 0 AND period < 7
      AND signer IN (
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y"
      )
    GROUP BY signer
  )
SELECT * FROM minting_rewards;
