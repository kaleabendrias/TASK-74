DROP INDEX IF EXISTS idx_rent_changes_status;

ALTER TABLE lodging_rent_changes
    DROP COLUMN IF EXISTS counterproposal_rent,
    DROP COLUMN IF EXISTS counterproposal_deposit,
    DROP COLUMN IF EXISTS counterproposed_by,
    DROP COLUMN IF EXISTS counterproposed_at;
