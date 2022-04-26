import React, { useState, useEffect } from 'react';
import { NFTStorage, File } from 'nft.storage';
import PropTypes from 'prop-types';
import Big from 'big.js';
import SignIn from './components/SignIn';
import Layout from './layout';
import NotFound from './components/404.jsx';
import Dashboard from './components/Dashboard.jsx';
import Collection from './components/Collection.jsx';
import Marketplace from './components/Marketplace.jsx';
import 'materialize-css/dist/css/materialize.css'
import './App.css';
import { Route, Routes } from 'react-router-dom'
var version = require('../package.json').version;
require('materialize-css');

const BOATLOAD_OF_GAS = Big(3).times(10 ** 13).toFixed();
const NFT_STORAGE_KEY = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJkaWQ6ZXRocjoweDc2NjhmOTg1OUYxYjZGNTRkRjk0YUMyM2U4RDk4OTE1QkQyQTg2MTciLCJpc3MiOiJuZnQtc3RvcmFnZSIsImlhdCI6MTY1MDUxNTU2NTA1MywibmFtZSI6Im5mdF9taW50In0.95zdADQVUy5zLpoe1kGV_ZiNvH2vlFIvC-h3CsMJEO8'
const VALID_EXTENSIONS = ["png", "jpg", "jpeg"];
const MAX_SIZE = 3;

const App = ({ contract, currentUser, nearConfig, wallet, provider, lastTransaction, error }) => {
  const [errorMessage, setErrorMessage] = useState('');
  const [message, setMessage] = useState('');

  const onNftMint = async (e) => {
    e.preventDefault();

    const { fieldset, file_chooser, title_prompt, description_prompt } = e.target.elements;
    
    fieldset.disabled = true;
    setErrorMessage('');
    
    let file = file_chooser.files[0];
    
    if (!file){
        setErrorMessage('No file was chosen.');
        fieldset.disabled = false;
        return
    }
    
    const uploadedFileExt = file.name
        .split('.')
        .pop()
        .toLowerCase()
    const isValidFileExt = VALID_EXTENSIONS.includes(uploadedFileExt)

    if (!isValidFileExt) {
        setErrorMessage(`Must upload a file of type: ${VALID_EXTENSIONS.join(' or ')}`);
        fieldset.disabled = false;
        return
    }
    
    const maxBytes = MAX_SIZE * 1024 * 1024;

    if (file.size > maxBytes) {
        setErrorMessage(`File size must be less than ${MAX_SIZE} MB.`);
        fieldset.disabled = false;
        return
    }
    
    const content = await file.arrayBuffer();
    const mime = file.type;    
    const result = await storeNFT(new File([content], file.name, { mime }), title_prompt.value, description_prompt.value);
    const imageUrl = result.data.image.href.replace("ipfs://", "https://ipfs.io/ipfs/");
    
    contract.nft_mint(
      { 
        token_owner_id: currentUser.accountId,
        token_metadata: {
                            title: title_prompt.value,
                            description: description_prompt.value,
                            media: imageUrl,
                            media_hash: 'RTBEMDBDNjZGODk1RTlEOEEyMTQzNjUyRjlCMUJGNEQ1MEU2NjQxNEM0RUI5NDQzMzdGRTcwMTk5NDFEMjkzQQ==', //fixed for now
                            copies: 1,
                            issued_at: Date.now().toString()
                        }
      },
      BOATLOAD_OF_GAS,
      Big('0.1').times(10 ** 24).toFixed()
    ).then((_) => {
      fieldset.disabled = false;
      title_prompt.value = '';
      description_prompt.value = '';
      title_prompt.focus();
    });
  };
  
  useEffect(() => {
      if (error){
        setMessage(decodeURI(error));
      }
      else if (lastTransaction) {          
          setMessage(`Successfully minted the NFT in transaction ${lastTransaction}`);
      }
      window.history.pushState({}, "", window.location.origin + window.location.pathname);
  }, [lastTransaction, error]);
  
  const signIn = () => {
    wallet.requestSignIn(
      {contractId: nearConfig.contractName, methodNames: [contract.nft_mint.name]}, //contract requesting access
      'NEAR Challenge #5 - Social Bounty Hunt', //optional name
      null, //optional URL to redirect to if the sign in was successful
      null //optional URL to redirect to if the sign in was NOT successful
    );
  };

  const signOut = () => {
    wallet.signOut();
    window.location.replace(window.location.origin + window.location.pathname);
  };

  const clearMessage = () => {
    setMessage('');
  };

  /**
    * Reads an image file from `image` and stores an NFT with the given name and description.
    * @param {string} image the file object of an image file
    * @param {string} name a name for the NFT
    * @param {string} description a text description for the NFT
    */
  const storeNFT = async (image, name, description) => {
      //TODO convert file to byte array
      // create a new NFTStorage client using our API key
      const nftstorage = new NFTStorage({ token: NFT_STORAGE_KEY })
      
       // call client.store, passing in the image & metadata
      return nftstorage.store({
          image,
          name,
          description,
      })
  }

  return (
    <Routes>
      <Route path="/" element={<Layout currentUser={currentUser} signIn={signIn} signOut={signOut} clearMessage={clearMessage} message={message}/>}>
        <Route index element={
          currentUser
            ? <Dashboard version={version} currentUser={currentUser}/>
            : <SignIn signIn={signIn}/>
        }/>
        <Route path="collection" element={
          currentUser
            ? <Collection onNftMint={onNftMint} currentUser={currentUser} errorMessage={errorMessage} contract={contract}/>
            : <SignIn signIn={signIn}/>
        }/>
        <Route path="marketplace" element={
          currentUser
            ? <Marketplace currentUser={currentUser} contract={contract}/>
            : <SignIn signIn={signIn}/>
        }/>
        <Route path="*" element={<NotFound/>}/>
      </Route>
    </Routes>
  );
}

App.propTypes = {
  contract: PropTypes.shape({
    nft_mint: PropTypes.func.isRequired
  }).isRequired,
  currentUser: PropTypes.shape({
    accountId: PropTypes.string.isRequired,
    balance: PropTypes.string.isRequired
  }),
  nearConfig: PropTypes.shape({
    contractName: PropTypes.string.isRequired
  }).isRequired,
  wallet: PropTypes.shape({
    requestSignIn: PropTypes.func.isRequired,
    signOut: PropTypes.func.isRequired
  }).isRequired
};

export default App;
