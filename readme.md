# Github Service to create "semi-open" repositories

## How it works

So currently the service works like this(was simplified in 2025):

- a single repo is montored for stars and unstars
- when a user stars the repo, an invite is sent to the user to join the organization
- the user accepts the invite and becomes a member of the organization
- the organisation members can see all the private repositories of the organization

Also there is an user bot that will monitor the user's starred repos and will comment to discusion if the user stars a preview repo and the invite is sent.

The user bot uses private API of GH because there are three kind of discussion respurces and the API only support the team discussions but not the org and repo discussions.

### Motivation

The motivation behind this app is to create a way to share code with people other than making a repo directly public.

### Notes

Creating invites has a very low rate limit, so you can invite less than a dozen people per day. This is a limitation of the GitHub API.

### Node Version Here

[https://github.com/flashsoft-eu/node-webhook-github-app-star-invite](https://github.com/flashsoft-eu/node-webhook-github-app-star-invite)

The node version might difffer from the one a bit because both have been updated since, intially they had same functionality.

### License

MIT License
