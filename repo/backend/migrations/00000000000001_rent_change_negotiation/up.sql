-- Extend lodging_rent_changes with counterproposal fields.
-- Valid status values are now: pending, countered, approved, rejected.

ALTER TABLE lodging_rent_changes
    ADD COLUMN counterproposal_rent    NUMERIC(12, 2),
    ADD COLUMN counterproposal_deposit NUMERIC(12, 2),
    ADD COLUMN counterproposed_by      UUID REFERENCES users(id),
    ADD COLUMN counterproposed_at      TIMESTAMPTZ;

CREATE INDEX idx_rent_changes_status ON lodging_rent_changes(status);
