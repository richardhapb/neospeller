<!-- PROJECT LOGO -->
<div align="center">
<h3 align="center">NeoSpeller</h3>
<br />
  
  ![ezgif-3d81514ffe89fb](https://github.com/user-attachments/assets/89765695-779f-4e70-b0d7-3689a78e878b)

<br />

  <p align="center">
    An awesome project to help you spell and grammar better! Pass your code with comments through stdin and receive a corrected version through stdout.
    <br />
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#contributing">Contributing</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## About The Project

A command-line tool written in Rust that uses LLM API to correct spelling and grammar in code comments.

This is useful for developers who want to improve their spelling and grammar in their code comments. That can connect to your editor through a plugin or extension, or you can use it as a standalone tool.

The app uses at the moment the OpenAI GPT-4 mini, and only send to the model the comments, avoiding the code, that is important for reducing the cost of the service. I will add more models in the future, and the user will be able to choose the model to use.

<!-- GETTING STARTED -->
## Getting Started

You need two simple things to get started, an API key from OpenAI and make sure you have Rust installed in your machine. If you don't have Rust installed, you can install it using [rustup](https://rustup.rs/).

### Installation

1. Clone the repo
   ```sh
   git clone https://github.com/richardhapb/neospeller.git
   ```

2. Execute the installation script (that will request your password to install the binary in /usr/local/bin)

   ```sh
   cd neospeller
   ./install.sh
   ```

3. Export the API key as an environment variable:
    ```sh
    export OPENAI_API_KEY="your-api-key"
    ```

<!-- USAGE EXAMPLES -->
## Usage

You can use the app in two ways, passing the code through stdin or passing a file as an argument. A needed argument is `--lang` that is the language of the comments in the code. The language is used to extract the comments from the code, and the code is not sent to the model.

```sh
neospeller --lang python < file.py
neospeller --lang javascript < file.js
```
or

```sh
cat file.py | neospeller --lang python
cat file.js | neospeller --lang javascript
```

---

You can redirect the output to a file:

```sh
neospeller --lang python < file.py > corrected_file.py
```

or

```sh
cat file.py | neospeller --lang python > corrected_file.py
```

Also, you can use this neovim plugin to correct the comments in the current buffer: [neospeller.nvim](https://github.com/richardhapb/neospeller.nvim)

Available languages and their respective codes:

- Python (python)
- Rust (rust)
- Go (go)
- JavaScript (javascript)
- CSS (css)
- C (c)
- Lua (lua)
- Bash (bash)
- Plain text (text)

<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git switch -c feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request




<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/richardhapb/neospeller.svg?style=for-the-badge
[contributors-url]: https://github.com/richardhapb/neospeller/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/richardhapb/neospeller.svg?style=for-the-badge
[forks-url]: https://github.com/richardhapb/neospeller/network/members
[stars-shield]: https://img.shields.io/github/stars/richardhapb/neospeller.svg?style=for-the-badge
[stars-url]: https://github.com/richardhapb/neospeller/stargazers
[issues-shield]: https://img.shields.io/github/issues/richardhapb/neospeller.svg?style=for-the-badge
[issues-url]: https://github.com/richardhapb/neospeller/issues
[license-shield]: https://img.shields.io/github/license/richardhapb/neospeller.svg?style=for-the-badge
[license-url]: https://github.com/richardhapb/neospeller/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/richard-hapb
[product-screenshot]: images/screenshot.png
