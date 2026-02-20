# Plugins and user-supplied code

`limabean` does not support externally provided plugins.  The intention is that all desired behaviour may be implemented by the end user in Clojure.

That said, there are a handful of internal plugins, as follows.

## Internal Plugins

### Implicit Prices

The existing plugin `beancount.plugins.implicit_prices` is built in.

### Auto Accounts

The existing plugin `beancount.plugins.auto_accounts` is built-in.

### Balance Rollup

As described above, the plugin `limabean.balance_rollup` modifies the behaviour of the `balance` directive.

## User-provided code

The user may provide their own Clojure code.  The environment variable `LIMABEAN_USER_CLJ` is a colon-separated list of Clojure source files, which are loaded in order, and made available in the REPL.

For example, see the [user-supplied `fy` function](../../examples/clj/user.clj) for a customized financial year filter.
