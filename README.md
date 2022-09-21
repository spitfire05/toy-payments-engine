# toy-payments-engine

## Notes, assumptions and considerations

* The type used to handle the transaction amounts is `f64`. In real application, probably something custom, less prone to rounding errors, should be used.
* Only deposit transactions can be disputed.