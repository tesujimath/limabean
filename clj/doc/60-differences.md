# Differences from OG Beancount and other Gotchas

_This is a very incomplete list.  It may grow over time._

## Balance assertions

A point of difference from classic Beancount is that balance assertions may be configured to assert the total for an account an all its subaccounts, using
the internal plugin `limabean.balance_rollup`.  For example, if a bank account holds multiple logical amounts, they may be tracked as subaccounts, without violating
balance assertions.

Padding is only ever performed on the actual account asserted in the balance directive, never on its subaccounts.

Unless the plugin is enabled, the default behaviour is not to do this.

## Duplicate includes

`limabean` allows the same file to be included multiple times, if and only if the tag and metadata context accumulated from push/pop tag/meta pragmas is identical.

Otherwise an error is output showing the differences in the include contexts.
