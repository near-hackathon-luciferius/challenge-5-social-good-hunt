import React from 'react';
import PropTypes from 'prop-types';
import { Button, TextInput } from 'react-materialize';

export default function Form({ onNftMint, errorMessage }) {
  return (
    <form onSubmit={onNftMint}>
      <fieldset id="fieldset">
        <p>Choose an image file below. Accepted are .png and .jpeg images only. The maximum file size is 3MB.</p>
        { errorMessage && <div className="error"> { errorMessage } </div> }
        <p className="highlight">
          <TextInput
            autoComplete="off"
            autoFocus
            id="title_prompt"
            className="name_input"
            label="The title of the image."
            required
          />
        </p>
        <p className="highlight">
          <TextInput
            autoComplete="off"
            autoFocus
            id="description_prompt"
            className="name_input"
            label="A short description of the image."
            required
          />
        </p>
        <p className="highlight">
          <input type="file" id="file_chooser" />
        </p>
        <Button type="submit" small
                tooltip="Mints the chosen image as a NFT.">
          Mint!
        </Button>
      </fieldset>
    </form>
  );
}

Form.propTypes = {
  onNftMint: PropTypes.func.isRequired,
  errorMessage: PropTypes.string.isRequired,
};