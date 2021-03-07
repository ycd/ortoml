import ortoml

as_dict = {"test": "val", "test1": {"another": "dict"}}

# convert dict to toml string
as_toml = ortoml.dumps(as_dict)


# convert toml string to dictionary
# as_dict_again = ortoml.loads(as_toml)
# assert as_dict_again == as_dict, "Dictionaries are not matching"
