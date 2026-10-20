Feature: Release phase order
  A release may only move to the phase that follows it.

  Scenario Outline: Phase transitions follow the release order
    Then <from> may move to <to>: <allowed>

    Examples:
      | from    | to      | allowed |
      | plan    | prepare | true    |
      | verify  | publish | true    |
      | plan    | publish | false   |
      | cut     | publish | false   |
