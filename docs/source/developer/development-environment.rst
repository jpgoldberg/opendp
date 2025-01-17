.. _development-environment:

Development Environment
=======================

Follow the steps below to get an OpenDP development environment set up, including the ability to run tests in both Rust and Python.

Install Rust
------------

Download Rust from the `Rust website`_.

.. _Rust website: https://www.rust-lang.org

Install Python
--------------

Download Python from the `Python website`_.

.. _Python website: https://www.python.org

Clone the OpenDP Repo
---------------------

If you don't have write access to the OpenDP repository, you will either need to request to join the organization or make a fork.
`The GitHub documentation explains forking <https://docs.github.com/en/get-started/quickstart/fork-a-repo>`_.

Clone the repo (or your fork) and change into the ``opendp`` directory that's created.

.. code-block:: bash

    git clone git@github.com:opendp/opendp.git
    cd opendp


If you have not `set up SSH <https://docs.github.com/en/authentication/connecting-to-github-with-ssh>`_, you can clone with https instead:

.. code-block:: bash

    git clone https://github.com/opendp/opendp.git


Building OpenDP
===============

Change to the ``rust`` directory before attempting a build, run the tests, and then return to the ``opendp`` directory.

.. code-block:: bash

    cd rust
    cargo build --features untrusted,bindings-python
    cargo test --features untrusted,bindings-python
    cd ..

Features are optional. The ``untrusted`` feature includes non-secure floating-point and contrib features like ``make_base_laplace``,
and the ``bindings-python`` feature updates the python bindings when you build.

Refer to the :ref:`developer-faq` if you run into compilation problems.

.. note::

    There is a more involved `setup guide <https://github.com/opendp/opendp/tree/main/rust/windows>`_ for Windows users.
    You can compromise to simple and vulnerable builds instead, by adding the ``--no-default-features`` flag to cargo commands.
    Be advised this flag disables GMP's exact float handling, as well as OpenSSL's secure noise generation.


Install Python Dependencies
---------------------------

Change to the ``python`` directory, create a Python virtual environment, activate it, install dependencies, and then install the Python OpenDP library itself.

.. code-block:: bash

    cd python

    # recommended. conda is just as valid
    python3 -m venv venv
    source venv/bin/activate

    pip install flake8 pytest
    pip install -e .

The `-e` flag is significant! It stands for "editable", meaning you only have to run this command once.

Testing Python
--------------

Run the tests from the ``python`` directory. 

.. code-block:: bash

    pytest -v

If pytest is not found, don't forget to activate your virtual environment.

Documentation
=============

The source for this documentation can be found in the "docs" directory at https://github.com/opendp/opendp

Building the Docs
-----------------

The docs are built using Sphinx and the steps are listed in the README in the "docs" directory.


Tooling
=======

There are many development environments that work with Rust. Here are a few:

* `VS Code <https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer>`_
* `Intellij IDEA <https://plugins.jetbrains.com/plugin/8182-rust>`_
* `Sublime <https://github.com/rust-lang/rust-enhanced>`_

Use whatever developer tooling you are comfortable with.


A few notes on VS Code:

* Be sure to install the `rust-analyzer <https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer>`_ plugin, not the rust plugin
* Open ``rust-analyzer``'s extension settings, search "features" and add ``"untrusted", "bindings-python"``
* Look for ``Problems`` in the bottom panel for live compilation errors as you work
* Other useful extensions are "Better Toml", "crates" and "LaTex Workshop".
* Starter json configurations:

.. raw:: html

   <details style="margin:-1em 0 2em 4em">
   <summary><a>Expand Me</a></summary>

Starter ``/.vscode/tasks.json``. 
These tasks can be used to directly build OpenDP.
`See also the VSCode documentation on tasks. <https://code.visualstudio.com/docs/editor/tasks>`_

.. code-block:: json

    {
        "version": "2.0.0",
        "tasks": [
            {
                "type": "cargo",
                "command": "build",
                "problemMatcher": [
                    "$rustc"
                ],
                "args": [
                    "--manifest-path=./rust/Cargo.toml"
                ],
                "group": "build",
                "label": "rust: cargo build",
                "presentation": {
                    "clear": true
                }
            },
            {
                "type": "cargo",
                "command": "build",
                "problemMatcher": [
                    "$rustc"
                ],
                "args": [
                    "--manifest-path=./rust/Cargo.toml",
                    "--features", "bindings-python untrusted"
                ],
                "group": "build",
                "label": "rust: cargo build ffi",
                "presentation": {
                    "clear": true
                }
            }
        ]
    }


Starter `settings.json` for LaTex Workshop. 
Access this file through the LaTex Workshop extension settings.
This configuration emits outputs into ``./out/``

.. code-block:: json

    {
        "latex-workshop.latex.outDir": "%DIR%/out/",
        "latex-workshop.latex.recipes": [
            {
                "name": "latexmk",
                "tools": [
                    "latexmk"
                ]
            }
        ],
        "latex-workshop.latex.tools": [
            {
                "name": "latexmk",
                "command": "latexmk",
                "args": [
                    "-synctex=1",
                    "-interaction=nonstopmode",
                    "-file-line-error",
                    "-recorder",
                    "-pdf",
                    "--shell-escape",
                    "-aux-directory=out",
                    "-output-directory=out",
                    "%DOC%"
                ]
            },
            {
                "name": "pdflatex",
                "command": "pdflatex",
                "args": [
                    "-synctex=1",
                    "-interaction=nonstopmode",
                    "-file-line-error",
                    "-aux-directory=out",
                    "-output-directory=out",
                    "%DOC%"
                ]
            }
        ],
        "latex-workshop.view.pdf.viewer": "tab"
    }

.. raw:: html

   </details>



A few notes on Intellij IDEA:

* Both the Intellij IDEA community edition and the CodeWithMe plugin are free
* Be sure to open the project at the root of the git repository
* Be sure to install the Python and Rust plugins for interactivity
* Be sure to "attach" the Cargo.toml in the red banner the first time you open a Rust source file
* Use run configurations to `build the rust library <https://plugins.jetbrains.com/plugin/8182-rust/docs/cargo-command-configuration.html#cargo-command-config>`_ and run tests
