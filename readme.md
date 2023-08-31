## Github App to create "semi-open" repositories

### How it works

So currently this app works like this:

- there is a map of of preview repos and their corresponding "real" repos
- the current implementation uses an app that installed on two organizations(a user counts as an organization too)
- app will monitor star and unstar events on preview repos
- when a preview repo is starred, the user will be invited to the "real" repo
- when a preview repo is unstarred, the user will be removed from the "real" repo

Also there is an user bot that will monitor the user's starred repos and will comment to discusion if the user stars a preview repo and the invite is sent.

The user bot uses private API of GH because there are three kind of discussion respurces and the API only support the team discussions but not the org and repo discussions.

### Motivation

The motivation behind this app is to create a way to share code with people other than making a repo directly public. 

### Notes

Creating invites has a very low rate limit, so you can invite less than a dozen people per day. This is a limitation of the GitHub API.

### Node Version Here

[https://github.com/flashsoft-eu/node-webhook-github-app-star-invite](https://github.com/flashsoft-eu/node-webhook-github-app-star-invite)

### Current development status

Working but bare.


