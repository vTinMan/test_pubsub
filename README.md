Experimental Publisher-Subsriber service based on HTTP, implemented on async_std without special web frameworks and libraries.

### Use case:

1) subscribers send GET requests to url and wait for response with data from publisher;
2) publisher sends POST request to same url with data that is then transported to response for subscribers.
