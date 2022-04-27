import React from 'react';
import { Button, TextInput } from 'react-materialize';

export default function Publish({onPublishDeed}) {
  return (
    <>
      <header>
                   <h1>Publish a social deed.</h1>
      </header>
      <form onSubmit={onPublishDeed}>
        <fieldset id="fieldset">
            <p>Describe your social deed and provide prove in form of an image or a gif.</p>
            <p>
                When you hit Publish it is possible that there are two transaction to sign.
                In this case your account is first registered with the contract in order for you to
                receive the DEED tokens. In the current implementation the registration cancles the
                actual publish which would need to be executed again after registration.
            </p>
            <p className="highlight">
            <TextInput
                autoComplete="off"
                autoFocus
                id="title_prompt"
                className="name_input"
                label="The title of your deed."
                required
            />
            </p>
            <p className="highlight">
            <TextInput
                autoComplete="off"
                id="description_prompt"
                className="name_input"
                label="A short description of the deed."
                required
            />
            </p>
            <p className="highlight">
            <TextInput
                autoComplete="off"
                id="proof_prompt"
                className="name_input"
                label="An URL to an image or of gif as prove of the deed."
                required
            />
            </p>
            <Button type="submit" small
                    tooltip="Publishes the social deed to the blockchain.">
            Publish
            </Button>
        </fieldset>
      </form>
    </>
  );
}