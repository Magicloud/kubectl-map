# Kubectl Map

This is a `kubectl` plugin that adds `map` command. The command takes a resource kind and a JSON path, applying the path to all objects of the resource kind, listing the result.

I made this tool becase `kubectl get --filter` only supports a few items to filter and it is even varies amount all resource kinds.
