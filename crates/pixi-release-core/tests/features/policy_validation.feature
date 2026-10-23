Feature: Release policy validation

  The authored policy is parsed and rejected before any release command performs I/O. A refusal
  names the exact key that is wrong, so the author does not have to guess.

  Scenario Outline: Validation reports the offending key
    Then policy case <case> yields <outcome>

    Examples:
      | case        | outcome            |
      | documented  | accepted           |
      | bad-target  | target.name        |
      | bad-format  | project.tag-format |
      | bad-source  | project.version-source |
