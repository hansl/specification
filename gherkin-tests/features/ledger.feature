@attr-2
Feature: Ledger

Scenario: Ledger can send tokens
  Given an identity id1
  And an identity id2
  And a symbol MFX
  Given id1 has 100 MFX
  And id2 has 0 MFX
  When id1 sends 50 MFX to id2
  Then the balance of id1 should be 50 MFX
  And the balance of id2 should be 50 MFX

Scenario: Ledger can list info
  Given a cbor params = {}
  When calling ledger.info with params into response1
  Then response response1 matches cddl type ledger-info
