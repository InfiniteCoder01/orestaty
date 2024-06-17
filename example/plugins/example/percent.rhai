// get first parameter from `params` array
let value = params[0];
// get key  value pair `label` from `hash` map
let label = hash["label"];

// compute the final string presentation
(value * 100).to_string() + label
