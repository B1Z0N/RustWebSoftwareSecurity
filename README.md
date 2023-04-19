# How to run

Using Docker:

1. Fill in the `.env` file with something like in `.env.exmpl`.
2. Run `docker-compose up --build`. 
3. Check your browser at `0.0.0.0:8085`

P.S. See [this](https://github.com/abiosoft/colima/issues/557) for MacOS troubleshooting.

# How come is it secure?

First of all I set up my virtual clean environment - Ubuntu 22.04 Arm version. I invoked the site using docker-compose, tested it a few times to check for basic functionality. I downloaded a few images and checked them.

## Owasp ZAP
After that I got to download and build OWASP ZAP and check the project with automated quick scans. Then I’ve got the alerts using traditional spider:

![image](https://user-images.githubusercontent.com/32279961/233066830-021f1561-de94-4cc3-9402-577b74999527.png)

And AJAX HtmlUnit spider took longer but expectedly got us more info:

![image](https://user-images.githubusercontent.com/32279961/233066856-038ee30c-1c9b-4ad0-9a73-643302b189d3.png)

## Fixes

I’ve decided to take a look at the code and fix some of bright issues with it:

1. First of all I’ve added proper handling of non-values - prepared some metamacros for that: null!, http_code!, define_http!, redir!, templ!
2. Then I’ve added transactions and rollback support to upload_post function.
3. Then I’ve implemented usage of minidom xml library instead of xmllint CLI app, and the use of MagicWand API instead of imagemagick CLI. That way it was less prone to command injection attacks.
4. Added shield from different attacks(mainly XSS). And added html_sanity check.
![image](https://user-images.githubusercontent.com/32279961/233066927-ac819f79-8a77-43e0-8eaa-14c6a09c61ee.png) 
5. Provided checks for SQL injections:
    * tokio_postgres [provides](https://www.google.com/url?q=https://github.com/sfackler/rust-postgres/issues/473&sa=D&source=editors&ust=1681908793819480&usg=AOvVaw26gw5jCVij4QYrb8bW96Xo) safety checks in its queries
    * And ‘ symbol escaping(with ‘’) in a search query.
6. Then I’ve proceeded with CSP to outline the sources from which the images, objects, stylesheets, scripts might come. See src/utils/fairings.
7. Finally I’ve provided anticsrf solution using rocket_csrf crate and it’s modification by [me](https://www.google.com/url?q=https://github.com/the10thWiz/rocket_csrf/pull/1&sa=D&source=editors&ust=1681908793820124&usg=AOvVaw3r6f_Nkc8d-gWwS3SfJoXj) and a few other participants of opensource community(mainly [CsrfForm](https://www.google.com/url?q=https://github.com/kotovalexarian/rocket_csrf/pull/5&sa=D&source=editors&ust=1681908793820406&usg=AOvVaw0axxmmLD9jkoJRmrPCOZpY) to autovalidate the csrf tokens). ((I have an idea of fully automating that with a powerful procedural macro)).
8. Switched to uuid-based id so that it was harder to predict private images. Added unique u16 to static image files naming, so that collisions were less likely and no one could accidentally(or not so) run and override someone else's image.
9. Ran cargo audit on my project to verify that no meaningful dependencies are corrupt with vulnerabilities(not to count net2 which is on rocket’s side). 
![image](https://user-images.githubusercontent.com/32279961/233067150-4eb9a893-371c-42b4-a396-52238ee9c1d3.png)
10. Created [my own docker image](https://www.google.com/url?q=https://hub.docker.com/repository/docker/b1z0n/magick-rust&sa=D&source=editors&ust=1681908793820989&usg=AOvVaw0XsLAlxPO0STmK8Y2V3x8x) that builds imagemagick with rust on debian.
11. Ran recently developed osv-scanner from google. Which showed minor errors(like net2 with is not vulnerability anymor and minor atty vulnerability). 
![image](https://user-images.githubusercontent.com/32279961/233067219-0a819e9a-9b1f-499b-ad4d-25e01106f7b9.png)
11. Ran recently developed osv-scanner from google. Which showed minor errors(like net2 with [is not](https://www.google.com/url?q=https://github.com/tokio-rs/mio/issues/1319&sa=D&source=editors&ust=1681908793821346&usg=AOvVaw1Oo1NEDA0kGK8lIuFhf8kq) vulnerability anymor and [minor](https://www.google.com/url?q=https://osv.dev/vulnerability/RUSTSEC-2021-0145&sa=D&source=editors&ust=1681908793821589&usg=AOvVaw2sckKEN4y-sIA_12BrbRnf) atty vulnerability). 
