# API Integration Reference

This file contains API integration details for Profile Pulse.

**Note**: This is a reference document, not a specification. It provides technical details for implementing profile fetching from social media platforms.

---

## Status

| Platform  | Status             | Method       | Rate Limit                     |
| --------- | ------------------ | ------------ | ------------------------------ |
| GitHub    | ✅ Recommended     | REST API     | 60/hr (unauth), 5000/hr (auth) |
| LinkedIn  | ⚠️ Fragile         | Web Scraping | ~100/day                       |
| Twitter/X | ⚠️ Paid Required   | API v2       | Varies by tier                 |
| Facebook  | ⚠️ App Review      | Graph API    | Varies                         |
| Instagram | ❌ Not Recommended | Unofficial   | Very restrictive               |

---

## GitHub Integration

### API Endpoint

```
GET https://api.github.com/users/{username}
```

### Response

```json
{
  "login": "octocat",
  "id": 1,
  "avatar_url": "https://avatars.githubusercontent.com/u/1?v=4",
  "name": "The Octocat",
  "email": "octocat@github.com",
  "location": "San Francisco",
  "company": "@github"
}
```

### Rate Limits

- Unauthenticated: 60 requests/hour per IP
- Authenticated: 5,000 requests/hour

### Implementation Notes

- Use `Authorization: token {token}` header for authenticated requests
- Check `X-RateLimit-Remaining` and `X-RateLimit-Reset` headers
- 404 on user not found, 403 on rate limit exceeded

---

## LinkedIn Integration

### Approach

- No public API available
- Must use web scraping
- Extract `og:image` meta tag from public profile page

### Rate Limits

- Conservative: ~100 requests/day
- 999 response = rate blocked

### Implementation Notes

- HTML structure changes frequently
- May require login for some profiles
- Respect robots.txt

---

## More Details

Full documentation: `docs/API_INTEGRATION.md` (793 lines)

---

**Status**: Reference Only  
**Last Updated**: 2026-01-15
